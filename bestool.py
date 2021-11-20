#! python3


from enum import Enum
import sys
from typing import List
import serial
import serial.tools.list_ports
from serial.tools import miniterm
import zlib
import click
import time
from datetime import datetime, timedelta

__author__ = "Ben V. Brown"
BES_BAUD = 921600


class BESMessageTypes(Enum):
    SYNC = 0x50
    START_PROGRAMMER = 0x53
    PROGRAMMER_RUNNING = 0x54
    PROGRAMMER_INIT = 0x60
    FLASH_COMMAND = 0x65
    ERASE_BURN_SART = 0x61
    FLASH_BURN_DATA = 0x62


class BESLink:
    """
    Wrapper class for communcations with the BES bootloader thing
    """

    @classmethod
    def wait_for_sync(cls, serial_port: serial.Serial):
        print(f"Waiting for sync on {serial_port.name}")
        exit_time = datetime.now() + timedelta(seconds=30)
        # Sync packet is {BE,50,00,03,00,00,01,ED}
        while datetime.now() < exit_time:
            packet = cls._read_packet(serial_port)
            if packet[1] == BESMessageTypes.SYNC.value:
                print("Reached sync 1 sending validation")
                break
        # Send out the confirmation message to stay in the bootloader
        resp_data = [0xBE, 0x50, 0x00, 0x01, 0x01, 0xEF]
        serial_port.write(resp_data)
        while datetime.now() < exit_time:
            packet = cls._read_packet(serial_port)
            if packet[1] == BESMessageTypes.SYNC.value:
                print("Programmer load stage 1")
                return
        raise Exception("Timeout")

    @classmethod
    def load_programmer_blob(cls, serial_port: serial.Serial):
        """
        Loading in the programmer blob
        """
        exit_time = datetime.now() + timedelta(seconds=30)
        cmd_prep_load_programmer = [
            0xBE,
            0x53,
            0x00,
            0x0C,
            0xDC,
            0x05,
            0x01,
            0x20,
            0xDC,
            0x32,
            0x01,
            0x00,
            0xC0,
            0xA7,
            0xE8,
            0x0C,
            0x76,
        ]
        # Send the prep command
        serial_port.write(cmd_prep_load_programmer)
        # wait for response
        while datetime.now() < exit_time:
            packet = cls._read_packet(serial_port)
            if packet[1] == BESMessageTypes.START_PROGRAMMER.value:
                print("Resp OK to start programmer load")
                break
        with open("programmer.bin", "r+b") as f:
            programmer_payload = f.read()
            serial_port.write(programmer_payload)
        # wait for response
        while datetime.now() < exit_time:
            packet = cls._read_packet(serial_port)
            if packet[1] == BESMessageTypes.PROGRAMMER_RUNNING.value:
                print("Resp to loading the programmer payload message")
                break
        cmd_programmer_start = [0xBE, 0x55, 0x01, 0x00, 0xEB]
        serial_port.write(cmd_programmer_start)
        while datetime.now() < exit_time:
            packet = cls._read_packet(serial_port)
            if packet[1] == BESMessageTypes.PROGRAMMER_INIT.value:
                print("Response ok to programmer start")
                break

    @classmethod
    def read_flash_info(cls, serial_port: serial.Serial):
        """
        Unknown if this _needs_ to be run

        """
        exit_time = datetime.now() + timedelta(seconds=30)
        print("starting reading flash id")
        cmd_get_flash_id = [0xBE, 0x65, 0x02, 0x01, 0x11, 0xC8]
        serial_port.write(cmd_get_flash_id)

        while datetime.now() < exit_time:
            packet = cls._read_packet(serial_port)
            if packet[1] == BESMessageTypes.FLASH_COMMAND.value:
                print(f"Flash info: ID {packet[5:8]}")
                break
        cmd_get_flash_unique_id = [0xBE, 0x65, 0x03, 0x01, 0x12, 0xC6]
        serial_port.write(cmd_get_flash_unique_id)

        while datetime.now() < exit_time:
            packet = cls._read_packet(serial_port)
            if packet[1] == BESMessageTypes.FLASH_COMMAND.value:
                print(f"Flash info: Unique ID {packet[5:]}")
                break

    @classmethod
    def program_binary_file(cls, serial_port: serial.Serial, filename: str):
        """
        Load the provided program in at the default locations

        """

        with open(filename, "r+b") as f:
            file_payload = f.read()
        file_length_raw = len(file_payload)
        # have to pad up to a multiple of 0x8000
        if file_length_raw % 0x8000 != 0:
            padding_len = 0x8000 - (file_length_raw % 0x8000)
            padding = [0xFF] * padding_len
            packed_file = file_payload + bytes(padding)

        file_length = len(file_payload)
        #
        start_address = 0x3C000000
        burn_start_msg = [
            0xBE,
            0x61,
            0x07,
            0x0C,
            0x00,
            0x00,
            0x00,
            0x3C,
            0x00,
            0x00,
            0x0D,
            0x00,
            0x00,
            0x80,
            0x00,
            0x0,
            0x04,
        ]
        burn_start_msg[4] = (start_address >> 0) & 0xFF
        burn_start_msg[5] = (start_address >> 8) & 0xFF
        burn_start_msg[6] = (start_address >> 16) & 0xFF
        burn_start_msg[7] = (start_address >> 24) & 0xFF
        burn_start_msg[8] = (file_length >> 0) & 0xFF
        burn_start_msg[9] = (file_length >> 8) & 0xFF
        burn_start_msg[10] = (file_length >> 16) & 0xFF
        burn_start_msg[11] = (file_length >> 24) & 0xFF
        # update checksum
        burn_start_msg[-1] = cls._calculate_message_checksum(burn_start_msg[0:-1])
        serial_port.write(burn_start_msg)
        exit_time = datetime.now() + timedelta(seconds=30)

        while datetime.now() < exit_time:
            packet = cls._read_packet(serial_port)
            if packet[1] == BESMessageTypes.ERASE_BURN_SART.value:
                print(f"Flash burn start returned {packet}")
                if packet[3] != 0x01:
                    raise Exception("Possible bad programming start?")
                break
        # Start splitting up the payload and sending it
        total_packets_to_send = len(packed_file) / 0x8000
        packets_waiting_ack = []
        seq = 0
        while len(packed_file) > 0:
            chunk = packed_file[0:0x8000]
            packed_file = packed_file[0x8000:]
            data_to_send = cls._create_burn_data_message(seq, chunk)
            print(f"Sending data chunk {seq}")
            serial_port.write(data_to_send)
            packets_waiting_ack.append(seq)
            if seq < 2:
                time.sleep(0.4)
            seq += 1
            while len(packets_waiting_ack) > 1:
                # Only allow two outstanding ones
                ack_seq = cls._wait_for_programming_ack(serial_port)
                if ack_seq in packets_waiting_ack:
                    packets_waiting_ack.remove(ack_seq)
                else:
                    raise Exception(f"Double ack for {ack_seq}")
        while len(packets_waiting_ack) > 0:
            # Only allow two outstanding ones
            print(f"Waiting for {packets_waiting_ack}")
            ack_seq = cls._wait_for_programming_ack(serial_port)
            if ack_seq in packets_waiting_ack:
                packets_waiting_ack.remove(ack_seq)
            else:
                raise Exception(f"Double ack for {ack_seq}")
        print("Sending done; sending commit")
        # Now send the final commit message
        commit_msg = [
            0xBE,
            0x65,
            0x08,
            0x09,
            0x22,
            0x00,
            0x00,
            0x00,
            0x3C,
            0x1C,
            0xEC,
            0x57,
            0xBE,
            0x50,
        ]
        commit_msg[5] = (start_address >> 0) & 0xFF
        commit_msg[6] = (start_address >> 8) & 0xFF
        commit_msg[7] = (start_address >> 16) & 0xFF
        commit_msg[8] = (start_address >> 24) & 0xFF
        commit_msg[-1] = cls._calculate_message_checksum(commit_msg[0:-1])
        serial_port.write(commit_msg)

        exit_time = datetime.now() + timedelta(seconds=30)
        while datetime.now() < exit_time:
            packet = cls._read_packet(serial_port)
            if packet[1] == BESMessageTypes.FLASH_COMMAND.value:
                if packet[2] == 0x08 and packet[3] == 0x01:
                    print("Done")
                    return
        raise Exception("Timed out finalising")

    @classmethod
    def _wait_for_programming_ack(cls, serial_port: serial.Serial) -> int:
        """
        Wait for an ack for programming
        """
        exit_time = datetime.now() + timedelta(seconds=30)
        while datetime.now() < exit_time:
            packet = cls._read_packet(serial_port)
            if packet[1] == BESMessageTypes.FLASH_BURN_DATA.value:
                sequence1 = packet[2] - 0xC1
                sequence2 = packet[5]

                print(f"Flash confirm {sequence1}/{sequence2}")
                if sequence2 == sequence1:
                    return sequence1
        raise Exception("Timeout waiting for programming ack")

    @classmethod
    def _create_burn_data_message(
        cls, sequence: int, data_payload: List[bytes]
    ) -> List[bytes]:
        """
        Creates the ready-to-send message to burn this chunk of data
        """
        chunk_size = len(data_payload)
        if chunk_size != 0x8000:
            raise Exception("Size not supported")
        template = [
            0xBE,
            0x62,
            0xC1,
            0x0B,
            0x00,
            0x80,
            0x00,
            0x00,
            0xAB,
            0x77,
            0x7F,
            0xF4,
            0x00,
            0x00,
            0x00,
            0xFE,
        ]
        template[2] = 0xC1 + sequence
        template[4] = chunk_size & 0xFF
        template[5] = (chunk_size >> 8) & 0xFF
        crc32_of_chunk = zlib.crc32(data_payload)
        template[8] = (crc32_of_chunk >> 0) & 0xFF
        template[9] = (crc32_of_chunk >> 8) & 0xFF
        template[10] = (crc32_of_chunk >> 16) & 0xFF
        template[11] = (crc32_of_chunk >> 24) & 0xFF
        template[12] = sequence
        template[15] = cls._calculate_message_checksum(template[0:-1])
        print("Tx H", bytes(template).hex(","))
        template.extend(data_payload)
        return template

    @classmethod
    def _read_packet(cls, port: serial.Serial) -> List[bytes]:
        """
        Try and read a bes packet in the timeout
        """
        packet = []
        packet_length = 3  # start at minimum

        while len(packet) < packet_length:
            data = port.read(size=1)
            data = data[0]
            if len(packet) == 0:
                if data == 0xBE:
                    packet.append(data)
            elif len(packet) == 2:
                packet.append(data)
                packet_length = cls._lookup_packet_length(packet[1], packet[2])
            else:
                packet.append(data)
        print("RX", bytes(packet).hex(","), packet)
        # Validate the checksum
        if not cls._validate_message_checksum(packet):
            raise Exception("Invalid message checksum")
        return packet

    @classmethod
    def _lookup_packet_length(cls, packet_id1: bytes, packet_id2: bytes):
        """
        Since they do not encode the length into the packet; we need to look them up manually
        This only stores the expected lengths for the messages coming from the MCU; for outgoing Tx messages that is left up to the sender functions
        """

        if packet_id1 == BESMessageTypes.SYNC.value:
            return 8
        if packet_id1 == BESMessageTypes.START_PROGRAMMER.value:
            return 6
        if packet_id1 == BESMessageTypes.PROGRAMMER_RUNNING.value:
            return 6
        if packet_id1 == BESMessageTypes.PROGRAMMER_INIT.value:
            return 11
        if packet_id1 == BESMessageTypes.FLASH_COMMAND.value:
            if packet_id2 == 2:
                return 9
            return 22
        if packet_id1 == BESMessageTypes.ERASE_BURN_SART.value:
            return 6
        if packet_id1 == BESMessageTypes.FLASH_BURN_DATA.value:
            return 8
        raise Exception(
            f"Unhandled packet length request for 0x{packet_id1:02x} / 0x{packet_id1:02x}"
        )

    @classmethod
    def _validate_message_checksum(cls, packet: List[bytes]) -> bool:
        """
        Validate the basic packet sum checksum for a message;
        this is actually just validate that all bytes sum to 0xFF (ignoring overflow)
        """
        chk = cls._calculate_message_checksum(packet[0:-1])
        return chk == packet[-1]

    @classmethod
    def _calculate_message_checksum(cls, packet: List[bytes]) -> bytes:
        """
        Calculates the checksum for this message and returns it
        """
        target = 0xFF
        sum = 0
        for b in packet:
            sum += b
            sum = sum & 0xFF
        return target - (sum)


# Spawn monitor on the port
def monitor(port: str):
    try:
        # Step on the args to stop them being parsed by miniterm
        sys.argv = ["besttool.py"]
        miniterm.main(
            default_port=port,
            default_baudrate=BES_BAUD,
        )
    except Exception as e:
        raise e


@click.group()
def cli():
    pass


@cli.command()
@click.argument("port_name")
def info(port_name):
    """"""
    print(f"Querying for info @ {port_name}")
    port = serial.Serial(port=port_name, baudrate=BES_BAUD, timeout=30)
    BESLink.wait_for_sync(port)
    BESLink.load_programmer_blob(port)
    BESLink.read_flash_info(port)
    port.close()


@cli.command()
@click.argument("filepath")
@click.argument("port_name")
def program(filepath, port_name):
    """"""
    print(f"beginning programming of {filepath} to device @ {port_name}")
    port = serial.Serial(port=port_name, baudrate=BES_BAUD, timeout=30)
    BESLink.wait_for_sync(port)
    BESLink.load_programmer_blob(port)
    BESLink.read_flash_info(port)
    BESLink.program_binary_file(port, filepath)
    port.close()


@cli.command()
@click.argument("filepath")
@click.argument("port")
def program_watch(filepath, port):
    """"""
    print(
        f"beginning programming of {filepath} to device @ {port} and then will drop into monitor"
    )
    monitor(port)


@cli.command()
def list_ports():
    """Lists available com ports"""
    print("Detected Ports")
    for port in serial.tools.list_ports.comports():
        print(port)


if __name__ == "__main__":
    cli()

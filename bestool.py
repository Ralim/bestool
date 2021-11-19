#! python3


from enum import Enum
from typing import List
import serial
import serial.tools.list_ports
from serial.tools import miniterm
import click
import sys
from datetime import datetime, timedelta

__author__ = "Ben V. Brown"
BES_BAUD = 921600


class BESMessageTypes(Enum):
    SYNC = 0x50
    START_PROGRAMMER = 0x53
    PROGRAMMER_RUNNING = 0x54
    PROGRAMMER_INIT = 0x60
    FLASH_INFO_RESP = 0x65


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
        cmd_prep_load_programmer = [0xBE, 0x53, 0x00, 0x0C, 0xDC, 0x05, 0x01, 0x20, 0xDC, 0x32, 0x01, 0x00, 0xC0, 0xA7, 0xE8, 0x0C, 0x76]
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
            if packet[1] == BESMessageTypes.FLASH_INFO_RESP.value:
                print(f"Flash info: ID {packet[5:8]}")
                break
        cmd_get_flash_unique_id = [0xBE, 0x65, 0x03, 0x01, 0x12, 0xC6]
        serial_port.write(cmd_get_flash_unique_id)

        while datetime.now() < exit_time:
            packet = cls._read_packet(serial_port)
            if packet[1] == BESMessageTypes.FLASH_INFO_RESP.value:
                print(f"Flash info: Unique ID {packet[5:]}")
                break

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
            print(f"0x{data:02X}")
            if len(packet) == 0:
                if data == 0xBE:
                    packet.append(data)
            elif len(packet) == 2:
                packet.append(data)
                packet_length = cls._lookup_packet_length(packet[1], packet[2])
            else:
                packet.append(data)
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
        if packet_id1 == BESMessageTypes.FLASH_INFO_RESP.value:
            if packet_id2 == 2:
                return 9
            return 22

        raise Exception(f"Unhandled packet length request for 0x{packet_id1:02x} / 0x{packet_id1:02x}")

    @classmethod
    def _validate_message_checksum(cls, packet: List[bytes]) -> bool:
        """
        Validate the basic packet sum checksum for a message;
        this is actually just validate that all bytes sum to 0xFF (ignoring overflow)
        """
        sum = 0
        for b in packet:
            sum += b
        sum = sum % 0xFF
        print(sum)
        # return sum == 1  # Need to investigate more
        return True


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
    print(f"beginning programming of {filepath} to device @ {port}")
    port = serial.Serial(port=port_name, baudrate=BES_BAUD, timeout=30)
    BESLink.wait_for_sync(port)
    BESLink.load_programmer_blob(port)
    BESLink.read_flash_info(port)
    port.close()


@cli.command()
@click.argument("filepath")
@click.argument("port")
def program_watch(filepath, port):
    """"""
    print(f"beginning programming of {filepath} to device @ {port} and then will drop into monitor")
    monitor(port)


@cli.command()
def list_ports():
    """Lists available com ports"""
    print("Detected Ports")
    for port in serial.tools.list_ports.comports():
        print(port)


if __name__ == "__main__":
    cli()

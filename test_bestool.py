from typing import List
import unittest

from bestool import BESLink


class MockSerial:
    __tx_data = []
    __rx_data = []
    name = "MockSerial"

    def add_tx(self, message: List[bytes]):
        self.__tx_data.extend(message)

    def get_rx_payloads(self) -> List[List[bytes]]:
        return self.__rx_data

    def read(self, size=1):
        if len(self.__tx_data) > 0:
            return [self.__tx_data.pop(0)]
        return None

    def write(self, data):
        self.__rx_data.append(data)


class TestBESLink(unittest.TestCase):
    def test_wait_for_sync(self):
        serial = MockSerial()
        serial.add_tx([0xBE, 0x50, 0x00, 0x03, 0x00, 0x00, 0x01, 0xED])
        serial.add_tx([0xBE, 0x50, 0x00, 0x03, 0x00, 0x00, 0x01, 0xED])
        BESLink.wait_for_sync(serial)
        # Assert that the payload was sent to it for the confirmation
        response_bytes = serial.get_rx_payloads()
        self.assertEqual(response_bytes, [[0xBE, 0x50, 0x00, 0x01, 0x01, 0xEF]])

    def test_load_programmer_blob(self):
        # Asserting that it will send the correct messages
        serial = MockSerial()

    def test_calc_checksum(self):
        samples = [
            [0xBE, 0x50, 0x00, 0x03, 0x00, 0x00, 0x01, 0xED],
            [0xBE, 0x50, 0x00, 0x01, 0x01, 0xEF],
            [0xBE, 0x53, 0x00, 0x01, 0x00, 0xED],
            [0xBE, 0x65, 0x02, 0x01, 0x11, 0xC8],
            [0xBE, 0x65, 0x03, 0x01, 0x12, 0xC6],
            [
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
            ],
            [
                0xBE,
                0x62,
                0xC2,
                0x0B,
                0x00,
                0x80,
                0x00,
                0x00,
                0x34,
                0x90,
                0x61,
                0xF9,
                0x01,
                0x00,
                0x00,
                0x73,
            ],
            [
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
                0x00,
                0x04,
            ],
            [
                0xBE,
                0x03,
                0x06,
                0x08,
                0x00,
                0xF0,
                0x0F,
                0x3C,
                0x00,
                0x10,
                0x00,
                0x00,
                0xE5,
            ],
            [
                0xBE,
                0x03,
                0x05,
                0x08,
                0x00,
                0xE0,
                0x0F,
                0x3C,
                0x00,
                0x10,
                0x00,
                0x00,
                0xF6,
            ],
        ]
        for sample in samples:
            chk = BESLink._calculate_message_checksum(sample[0:-1])
            self.assertEqual(chk, sample[-1], msg=f"{sample} - {chk:02X}")

    def test_generate_chunk(self):
        expected_header_data = [
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
        with open("chunk1.bin", "r+b") as f:
            payload = f.read()
        messsage_out = BESLink._create_burn_data_message(0, payload)
        self.assertEqual(
            messsage_out[0 : len(expected_header_data)], expected_header_data
        )
        self.assertEqual(bytes(messsage_out[len(expected_header_data) :]), payload)

        expected_header_data = [
            0xBE,
            0x62,
            0xC2,
            0x0B,
            0x00,
            0x80,
            0x00,
            0x00,
            0x34,
            0x90,
            0x61,
            0xF9,
            0x01,
            0x00,
            0x00,
            0x73,
        ]
        with open("chunk2.bin", "r+b") as f:
            payload = f.read()
        messsage_out = BESLink._create_burn_data_message(1, payload)
        self.assertEqual(
            messsage_out[0 : len(expected_header_data)], expected_header_data
        )
        self.assertEqual(bytes(messsage_out[len(expected_header_data) :]), payload)

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

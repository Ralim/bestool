#[cfg(test)]
mod tests {
    use crate::beslink::BES_PROGRAMMING_BAUDRATE;
    use serialport::{ClearBuffer, DataBits, FlowControl, Parity, SerialPort, StopBits};
    use std::io::{Read, Write};
    use std::time::Duration;

    pub struct FakeSerialPort {
        baud: u32,
    }

    impl Read for FakeSerialPort {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            todo!()
        }
    }

    impl Write for FakeSerialPort {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            todo!()
        }

        fn flush(&mut self) -> std::io::Result<()> {
            todo!()
        }
    }

    impl SerialPort for FakeSerialPort {
        fn name(&self) -> Option<String> {
            Some("Faux".to_owned())
        }

        fn baud_rate(&self) -> serialport::Result<u32> {
            Ok(BES_PROGRAMMING_BAUDRATE)
        }

        fn data_bits(&self) -> serialport::Result<DataBits> {
            Ok(DataBits::Eight)
        }

        fn flow_control(&self) -> serialport::Result<FlowControl> {
            Ok(FlowControl::None)
        }

        fn parity(&self) -> serialport::Result<Parity> {
            Ok(Parity::None)
        }

        fn stop_bits(&self) -> serialport::Result<StopBits> {
            Ok(StopBits::One)
        }

        fn timeout(&self) -> Duration {
            Duration::from_millis(1000)
        }

        fn set_baud_rate(&mut self, baud_rate: u32) -> serialport::Result<()> {
            self.baud = baud_rate;
            Ok(())
        }

        fn set_data_bits(&mut self, _data_bits: DataBits) -> serialport::Result<()> {
            unimplemented!()
        }

        fn set_flow_control(&mut self, _flow_control: FlowControl) -> serialport::Result<()> {
            unimplemented!()
        }

        fn set_parity(&mut self, _parity: Parity) -> serialport::Result<()> {
            unimplemented!()
        }

        fn set_stop_bits(&mut self, _stop_bits: StopBits) -> serialport::Result<()> {
            unimplemented!()
        }

        fn set_timeout(&mut self, _timeout: Duration) -> serialport::Result<()> {
            unimplemented!()
        }

        fn write_request_to_send(&mut self, _level: bool) -> serialport::Result<()> {
            unimplemented!()
        }

        fn write_data_terminal_ready(&mut self, _level: bool) -> serialport::Result<()> {
            unimplemented!()
        }

        fn read_clear_to_send(&mut self) -> serialport::Result<bool> {
            unimplemented!()
        }

        fn read_data_set_ready(&mut self) -> serialport::Result<bool> {
            unimplemented!()
        }

        fn read_ring_indicator(&mut self) -> serialport::Result<bool> {
            unimplemented!()
        }

        fn read_carrier_detect(&mut self) -> serialport::Result<bool> {
            unimplemented!()
        }

        fn bytes_to_read(&self) -> serialport::Result<u32> {
            todo!()
        }

        fn bytes_to_write(&self) -> serialport::Result<u32> {
            unimplemented!()
        }

        fn clear(&self, _buffer_to_clear: ClearBuffer) -> serialport::Result<()> {
            unimplemented!()
        }

        fn try_clone(&self) -> serialport::Result<Box<dyn SerialPort>> {
            unimplemented!()
        }

        fn set_break(&self) -> serialport::Result<()> {
            unimplemented!()
        }

        fn clear_break(&self) -> serialport::Result<()> {
            unimplemented!()
        }
    }
}

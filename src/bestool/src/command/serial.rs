use std::io::Result;

use clap::Subcommand;

mod enumerate_serial_ports;
mod monitor;

#[derive(Subcommand, Debug)]
#[command(infer_subcommands = true)]
pub enum Cmd {
    EnumerateSerialPorts,
    Monitor, // Add `(monitor::Cmd)` here.
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        match self {
            Self::EnumerateSerialPorts => enumerate_serial_ports::run(),
            Self::Monitor => monitor::run(),
        }
    }
}

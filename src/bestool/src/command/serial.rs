use std::io::Result;

use clap::Subcommand;

pub(crate) mod enumerate_serial_ports;
pub(crate) mod monitor;

#[derive(Subcommand, Debug)]
#[command(infer_subcommands = true)]
pub(crate) enum Cmd {
    EnumerateSerialPorts,
    Monitor, // Add `(monitor::Cmd)` here.
}

impl Cmd {
    pub(crate) fn run(self) -> Result<()> {
        match self {
            Self::EnumerateSerialPorts => enumerate_serial_ports::run(),
            Self::Monitor => monitor::run(),
        }
    }
}

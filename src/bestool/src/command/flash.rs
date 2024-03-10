use std::io::Result;
use clap::Subcommand;

pub(crate) mod write_flash;
pub(crate) mod read_flash;

#[derive(Subcommand, Debug)]
#[command(infer_subcommands = true)]
pub(crate) enum Cmd {
    WriteFlash, // Use `(write_flash::Cmd)` (with args `--monitor`)
    ReadFlash,  // Use `(read_flash::Cmd)`
}

impl Cmd {
    pub(crate) fn run(self) -> Result<()> {
        match self {
            Self::WriteFlash => write_flash::run(),
            Self::ReadFlash => read_flash::run(),
        }
    }
}

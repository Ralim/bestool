use clap::Subcommand;
use std::io::Result;

mod read_flash;
mod write_flash;

#[derive(Subcommand, Debug)]
#[command(infer_subcommands = true)]
pub enum Cmd {
    WriteFlash, // Use `(write_flash::Cmd)` (with args `--monitor`)
    ReadFlash,  // Use `(read_flash::Cmd)`
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        match self {
            Self::WriteFlash => write_flash::run(),
            Self::ReadFlash => read_flash::run(),
        }
    }
}

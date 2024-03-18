use clap::Subcommand;
use std::io::Result;

mod flash;
mod serial;

#[derive(Subcommand)]
#[command(infer_subcommands = true)]
pub enum BestoolCmd {
   #[command(flatten)]
   Flash(flash::Cmd),

    #[command(flatten)]
    Serial(serial::Cmd),
}

impl BestoolCmd {
    pub fn run(self) -> Result<()> {
        match self {
            Self::Flash(flash) => flash.run(),
            Self::Serial(serial) => serial.run(),
        }
    }
}

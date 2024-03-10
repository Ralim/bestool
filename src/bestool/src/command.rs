use std::io::Result;
use clap::Subcommand;

pub(crate) mod flash;
pub(crate) mod serial;

#[derive(Subcommand)]
#[command(infer_subcommands = true)]
pub(crate) enum BestoolCmd {
    #[command(flatten)]
    Flash(flash::Cmd),

    #[command(flatten)]
    Serial(serial::Cmd),
}

impl BestoolCmd {
    pub(crate) fn run(self) -> Result<()> {
        match self {
            Self::Flash(flash) => flash.run(),
            Self::Serial(serial) => serial.run(),
        }
    }
}

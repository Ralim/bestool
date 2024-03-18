#![warn(clippy::all)]

use clap::Parser;
mod command;
use command::BestoolCmd;
use std::io::Result;

const VERSION: &str = env!("CARGO_PKG_VERSION");

static HELP_TEMPLATE: &str = "\
{before-help}{name} {version}
{author}
{about}

{usage-heading}
  {usage}

{all-args}{after-help}";

#[derive(Parser)]
#[command(
    author = "Ben V. Brown <ralim@ralimtek.com>",
    version = VERSION,
    help_template = HELP_TEMPLATE,
)]
struct Bestool {
    #[command(subcommand)]
    bestool: BestoolCmd,
}

impl Bestool {
    fn run(self) -> Result<()> {
        self.bestool.run()
    }
}

fn main() -> Result<()> {
    Bestool::parse().run()
}

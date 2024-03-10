#![warn(clippy::pedantic, clippy::nursery)]

use clap::Parser;
pub(crate) mod command;
use command::BestoolCmd;
use std::io::Result;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

static HELP_TEMPLATE: &'static str = "\
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
    Ok(Bestool::parse().run()?)
}

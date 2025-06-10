#[cfg(feature = "std")]
extern crate std;

use clap::Parser;

use super::{execute_command, CliError, Commands};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

pub fn run() -> Result<(), CliError> {
    let cli = Cli::parse();
    execute_command(&cli.command)
}

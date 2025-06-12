#[cfg(feature = "std")]
extern crate std;

use clap::Parser;

use super::{execute_command, Cli, CliError};

pub fn run() -> Result<(), CliError> {
    let cli = Cli::parse();
    execute_command(&cli.command)
}

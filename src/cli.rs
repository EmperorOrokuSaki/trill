use clap::Parser;
use eyre::Result;

use crate::args::{CliArgs, Commands};
use crate::commands::{inspect, profile};

pub struct Cli {
    args: CliArgs,
}

impl Cli {
    pub fn new(args: CliArgs) -> Self {
        Self { args }
    }

    pub fn parse() -> Self {
        Self {
            args: CliArgs::parse(),
        }
    }

    pub async fn exec(self) -> Result<()> {
        match self.args.commands {
            Commands::Inspect(args) => inspect::exec(args).await,
            Commands::Profile(args) => profile::exec(args).await,
        }
    }
}

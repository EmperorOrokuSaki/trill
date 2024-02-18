use anyhow::Result;

use crate::args::{CliArgs, Commands};
use crate::commands::{inspect, profile};

pub struct Cli {
    args: CliArgs,
}

impl Cli {
    pub fn new(args: CliArgs) -> Self {
        Self { args }
    }

    pub fn exec(self) -> Result<()> {
        match self.args.commands {
            Commands::Inspect(_) => inspect::exec(),
            Commands::Profile(_) => profile::exec(),
        }
    }
}

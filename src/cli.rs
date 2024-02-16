use crate::args::{Args, Commands};
use crate::commands::{inspect, profile};

struct Cli {
    args: Args
}

impl Cli {
    pub fn new(args: Args) -> Self {
        Self { Args }
    }

    pub fn exec(self) -> Result<()> {
        match self.args.commands {
            Commands::Inspect => inspect::exec(),
            Commands::Profile => profile::exec(),
        }
    }
}
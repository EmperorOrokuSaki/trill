use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)] // Read from `Cargo.toml`
pub struct CliArgs {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Inspect tool for examining transactions and blocks
    Inspect(InspectArgs),
    /// Profile tool for making profiles of transactions and blocks
    Profile(ProfileArgs),
}

#[derive(Args)]
pub struct InspectArgs {
    /// The transaction hash to inspect
    #[arg(short, long)]
    transaction: Option<String>,
    /// The block number to inspect
    #[arg(short, long)]
    block: Option<u64>,
}

#[derive(Args)]
pub struct ProfileArgs {
    /// The address to make a profile for
    #[arg(short, long)]
    address: String,
    /// Brief profiling of the address
    #[arg(short, long)]
    brief: bool,
}

use clap::{Args, Parser, Subcommand};
use ethers::core::types::{Address, TxHash};

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

#[derive(Args, Debug)]
pub struct InspectArgs {
    /// The transaction hash to inspect
    #[arg(short, long)]
    pub transaction: Option<TxHash>,
    /// The block number to inspect
    #[arg(short, long)]
    pub block: Option<u64>,
}

#[derive(Args)]
pub struct ProfileArgs {
    /// The address to make a profile for
    #[arg(short, long)]
    pub address: Address,
    /// Brief profiling of the address
    #[arg(short, long)]
    pub brief: bool,
}

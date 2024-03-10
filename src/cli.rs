use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Transaction hash
    #[arg(short, long)]
    pub transaction: String,
}

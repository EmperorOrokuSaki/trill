use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Transaction hash
    #[arg(short, long)]
    pub transaction: String,
}

use clap::Parser;

static ANVIL_DEFAULT_RPC : &str = "http://127.0.0.1:8545";

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Transaction hash
    #[arg(short, long)]
    pub transaction: String,
    /// Number of operations to process every second
    #[arg(short, long, default_value_t = 4.0)]
    pub speed: f64,
    /// The JSON-RPC endpoint URL
    #[arg(short, long, default_value_t = ANVIL_DEFAULT_RPC.to_string())]
    pub rpc: String
}

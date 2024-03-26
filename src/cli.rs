use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Transaction hash
    #[arg(short, long)]
    pub transaction: String,
    /// Number of operations to process every second
    #[arg(short, long, default_value_t = 4.0)]
    pub speed: f64
}

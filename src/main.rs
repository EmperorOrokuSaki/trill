use clap::Parser;
use ethers::providers::{Http, Middleware, Provider};
use log::{info, LevelFilter};
use trill::args::CliArgs;
use trill::cli::Cli;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Initialize the logger with the desired log level
    env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .init();

    // Initialize a new Http provider
    let rpc_url = "";
    let provider = Provider::try_from(rpc_url)?;
    let args = CliArgs::parse();
    let cli = Cli::new(args);
    if let Err(err) = cli.exec() {
        eprintln!("Error {}", err);
        std::process::exit(1);
    }
    Ok(())
}

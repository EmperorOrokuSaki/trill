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

    let provider: Provider<Http>;

    if let Some(rpc) = std::env::var("RPC_HTTP").ok() {
        provider = Provider::try_from(rpc)?;
    } else {
        println!("PATH environment variable is not set");
    }

    let args = CliArgs::parse();
    let cli = Cli::new(args);

    if let Err(err) = cli.exec().await {
        eprintln!("Error {}", err);
        std::process::exit(1);
    }
    Ok(())
}

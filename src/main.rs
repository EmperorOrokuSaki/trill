use clap::Parser;
use log::LevelFilter;
use trill::args::CliArgs;
use trill::cli::Cli;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Initialize the logger with the desired log level
    env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .init();

    let args = CliArgs::parse();
    let cli = Cli::new(args);

    if let Err(err) = cli.exec().await {
        eprintln!("Error {}", err);
        std::process::exit(1);
    }
    Ok(())
}

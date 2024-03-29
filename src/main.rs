mod app;
mod cli;
mod log;
mod provider;
mod render;
mod state;
mod tui;

use std::str::FromStr;

use alloy::primitives::TxHash;
use app::App;
use clap::Parser;
use cli::Cli;
use color_eyre::Result;
use log::initialize_logging;

#[tokio::main]
async fn main() -> Result<()> {
    initialize_logging()?;
    let cli = Cli::parse();
    let transaction = TxHash::from_str(cli.transaction.as_str())?;
    let fps = cli.fps;
    let iteration = cli.iteration;
    let rpc = cli.rpc;
    App::default().run(transaction, fps, iteration, rpc).await?;
    Ok(())
}

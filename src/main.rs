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
use color_eyre::{eyre, Result};
use log::initialize_logging;
use state::AppState;

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    initialize_logging()?;
    let cli = Cli::parse();
    let transactions: Vec<TxHash> = cli
        .transaction
        .iter()
        .map(|transaction| TxHash::from_str(transaction.as_str()).unwrap())
        .collect();
    let fps = cli.fps;
    let iteration = cli.iteration;
    let rpc = cli.rpc;
    let mut app_state = AppState::default();
    app_state.init(&rpc, transactions).await?;
    App::default().run(&mut app_state, fps, iteration).await?;
    Ok(())
}

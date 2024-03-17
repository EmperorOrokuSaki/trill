mod app;
mod cli;
mod error;
mod provider;
mod render;
mod state;
mod tui;

use std::{ops::Deref, str::FromStr};

use alloy::primitives::{FixedBytes, TxHash};
use app::App;
use clap::Parser;
use cli::Cli;
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    error::install_hooks()?;
    let cli = Cli::parse();
    let transaction = TxHash::from_str(cli.transaction.as_str())?;
    App::default().run(transaction).await?;
    Ok(())
}

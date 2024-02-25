use std::io;
mod provider;
mod tui;
mod app;

use app::App;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};

use alloy::{
    primitives::{address, U256},
    providers::{Provider, ProviderBuilder},
};
use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // error::install_hooks()?;
    App::default().run().await?;
    Ok(())
}

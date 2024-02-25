use std::io;

use app::App;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};

mod app;
mod error;
mod tui;

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
    error::install_hooks()?;
    let mut terminal = tui::init()?;
    App::default().run(&mut terminal)?;
    tui::restore()?;
    Ok(())
}

use std::io;

use app::App;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};

mod error;
mod tui;
mod app;

use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};

fn main() -> Result<()> {
    error::install_hooks()?;
    let mut terminal = tui::init()?;
    App::default().run(&mut terminal)?;
    tui::restore()?;
    Ok(())
}

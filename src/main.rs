mod app;
mod cli;
mod error;
mod provider;
mod render;
mod state;
mod tui;

use app::App;
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    error::install_hooks()?;
    App::default().run().await?;
    Ok(())
}

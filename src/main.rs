mod app;
mod provider;
mod render;
mod state;
mod tui;
mod error;
use app::App;

use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    error::install_hooks()?;
    App::default().run().await?;
    Ok(())
}

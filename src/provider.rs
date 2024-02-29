use alloy::{
    providers::{ProviderBuilder, RootProvider},
    transports::BoxTransport,
};
use color_eyre::eyre::{self, eyre};

pub struct HTTPProvider {
    pub provider: RootProvider<BoxTransport>,
}

impl HTTPProvider {
    pub async fn new() -> Result<RootProvider<BoxTransport>, eyre::Error> {
        match ProviderBuilder::new()
            .on_builtin("http://127.0.0.1:8545")
            .await
        {
            Ok(provider) => return Ok(provider),
            Err(error) => return Err(eyre!(error)),
        }
    }
}

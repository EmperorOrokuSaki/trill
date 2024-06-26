use alloy::{
    providers::{ProviderBuilder, RootProvider},
    transports::BoxTransport,
};
use color_eyre::eyre::{self, eyre};

pub struct HTTPProvider {
    pub provider: RootProvider<BoxTransport>,
}

impl HTTPProvider {
    pub async fn init(rpc: &str) -> Result<RootProvider<BoxTransport>, eyre::Error> {
        match ProviderBuilder::new().on_builtin(rpc).await {
            Ok(provider) => Ok(provider),
            Err(error) => Err(eyre!(error)),
        }
    }
}

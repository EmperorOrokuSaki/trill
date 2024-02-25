use alloy::{providers::{ProviderBuilder, RootProvider}, transports::BoxTransport};
use color_eyre::eyre::{self, eyre};

pub struct HTTPProvider {
    pub provider: RootProvider<BoxTransport>
}

impl HTTPProvider {
    pub async fn new() -> Result<RootProvider<BoxTransport>, eyre::Error> {
        if let Some(rpc) = std::env::var("RPC_HTTP").ok() {
            match ProviderBuilder::new().on_builtin(&rpc).await {
                Ok(provider) => return Ok(provider),
                Err(error) => return Err(eyre!(error))
            }
        }
        Err(eyre!("the RPC_HTTP env var is not set"))
    }
}
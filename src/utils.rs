use ethers::prelude::{Provider, Http};
use eyre::{Result, eyre};

pub fn create_provider() -> Result<Provider<Http>> {
    if let Some(rpc) = std::env::var("RPC_HTTP").ok() {
        return match Provider::try_from(rpc) {
            Ok(provider) => Ok(provider),
            Err(error) => Err(error.into())
        };
    }
    Err(eyre!("Environment variable `RPC_HTTP` must be set and accessible."))
}
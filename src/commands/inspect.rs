use ethers::{
    core::types::TxHash,
    prelude::Middleware,
};
use eyre::{eyre, Result};

use crate::utils::create_provider;
use crate::args::InspectArgs;

pub async fn exec(args: InspectArgs) -> Result<()> {
    match (args.transaction, args.block) {
        (Some(transaction), _) => inspect_transaction(transaction).await,
        (_, Some(block)) => inspect_block(block).await,
        _ => Err(eyre!("At least one option should be provided.")),
    }
}

pub async fn inspect_transaction(tx_id: TxHash) -> Result<()> {
    let provider = create_provider()?;
    let a = provider.get_transaction(tx_id).await?;
    log::info!("{:?}", a);
    Ok(())
}

pub async fn inspect_block(block_number: u64) -> Result<()> {
    let provider = create_provider().unwrap();
    let a = provider.get_block(block_number).await?.unwrap();
    log::info!("{:?}", a.author);
    Ok(())
}

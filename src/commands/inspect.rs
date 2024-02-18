use ethers::{
    core::types::TxHash,
    prelude::{Middleware, Provider},
};
use eyre::{eyre, Result};

use crate::args::InspectArgs;

pub async fn exec(args: InspectArgs) -> Result<()> {
    match (args.transaction, args.block) {
        (Some(transaction), _) => inspectTransaction(transaction).await,
        (_, Some(block)) => inspectBlock(block).await,
        _ => Err(eyre!("At least one option should be provided.")),
    }
}

pub async fn inspectTransaction(txId: TxHash) -> Result<()> {
    let rpc_url = "";
    let provider = Provider::try_from(rpc_url)?;
    let a = provider.get_transaction(txId).await?;
    log::info!("{:?}", a);
    Ok(())
}

pub async fn inspectBlock(blockNumber: u64) -> Result<()> {
    let rpc_url = "";
    let provider = Provider::try_from(rpc_url)?;
    let a = provider.get_block(blockNumber).await?.unwrap();
    log::info!("{:?}", a.author);
    Ok(())
}

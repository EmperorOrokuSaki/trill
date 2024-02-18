use ethers::prelude::transaction;
use eyre::{eyre, Result};

use crate::args::InspectArgs;

pub fn exec(args: InspectArgs) -> Result<()> {
    log::info!("here");
    if let Some(transaction) = args.transaction.as_ref() {
        log::info!("here 1");
        inspectTransaction(transaction)
    } else if let Some(block) = args.block {
        log::info!("here 2");
        inspectBlock(block)
    } else {
        log::info!("here 3");
        Err(eyre!("At least one option should be provided."))
    }
}

pub fn inspectTransaction(txId: &String) -> Result<()> {
    Ok(())
}

pub fn inspectBlock(blockNumber: u64) -> Result<()> {
    Ok(())
}

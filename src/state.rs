use alloy::{
    primitives::fixed_bytes,
    providers::Provider,
    rpc::types::{eth::Transaction, trace::{
        self,
        geth::{GethDebugTracingOptions, GethDefaultTracingOptions, GethTrace, StructLog},
    }},
};
use color_eyre::eyre;

use crate::provider;

pub struct AppState {
    pub slots: Vec<SlotStatus>,
    pub indexed_slots_count: u64,
    pub next_operation: u64,
    pub next_slot_status: SlotStatus,
    pub operation_codes: Vec<Operations>,
    pub raw_data: Vec<StructLog>,
    pub initialized: bool,
    pub transaction: Transaction,
    pub transaction_sucess: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            slots: vec![],
            indexed_slots_count: 0,
            next_operation: 0,
            next_slot_status: SlotStatus::INIT,
            raw_data: vec![],
            initialized: false,
            operation_codes: vec![],
            transaction: Transaction::default(),
            transaction_sucess: false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operations {
    MSTORE,
    MSTORE8,
    MLOAD,
    CALLDATACOPY,
    RETURNDATACOPY,
}

impl Operations {
    pub fn text(&self) -> &'static str {
        match self {
            Operations::MSTORE => "MSTORE",
            Operations::MSTORE8 => "MSTORE8",
            Operations::MLOAD => "MLOAD",
            Operations::CALLDATACOPY => "CALLDATACOPY",
            Operations::RETURNDATACOPY => "RETURNDATACOPY",
        }
    }
    pub fn from_text(op: &str) -> Result<Self, eyre::Error> {
        match op {
            "MSTORE" => return Ok(Operations::MSTORE),
            "MSTORE8" => return Ok(Operations::MSTORE8),
            "MLOAD" => return Ok(Operations::MLOAD),
            "CALLDATACOPY" => return Ok(Operations::CALLDATACOPY),
            "RETURNDATACOPY" => return Ok(Operations::RETURNDATACOPY),
            _ => return Err(eyre::eyre!("Unidentified op code.")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlotStatus {
    INIT,
    EMPTY,
    ACTIVE,
    READING,
    WRITING,
}

impl SlotStatus {
    pub fn text(&self) -> &'static str {
        match self {
            SlotStatus::INIT => "Initializing",
            SlotStatus::EMPTY => "Empty",
            SlotStatus::ACTIVE => "Active",
            SlotStatus::READING => "Reading",
            SlotStatus::WRITING => "Writing",
        }
    }
}

impl AppState {
    async fn initialize(&mut self) -> Result<(), eyre::Error> {
        let provider = provider::HTTPProvider::new().await?;
        let tx_hash = fixed_bytes!("cd3d9bba59cb634070a0b84bf333c97daed0eb6244929f3ba27b847365bbe546");
        let transaction_result = provider.get_transaction_by_hash(tx_hash).await?;
        self.transaction = transaction_result;

        let opts = GethDebugTracingOptions {
            config: GethDefaultTracingOptions {
                enable_memory: Some(true),
                disable_memory: None,
                disable_stack: Some(true),
                disable_storage: Some(true),
                enable_return_data: Some(true),
                disable_return_data: Some(false),
                debug: None,
                limit: None,
            },
            tracer: None,
            tracer_config: trace::geth::GethDebugTracerConfig(serde_json::Value::Null),
            timeout: None,
        };
        
        let result = provider
            .debug_trace_transaction(
                tx_hash,
                opts,
            )
            .await?;

        match result {
            GethTrace::JS(context) => {
                self.transaction_sucess = !serde_json::from_value(context["failed"].clone())?;
                self.raw_data = serde_json::from_value(context["structLogs"].clone())?;
                let max_memory_length = self
                    .raw_data
                    .iter()
                    .filter(|operation| operation.memory.is_some())
                    .map(|operation| operation.memory.as_ref().unwrap().len())
                    .max()
                    .unwrap_or(0);
                self.slots = vec![SlotStatus::EMPTY; max_memory_length];
            }
            _ => (),
        }
        self.initialized = true;

        Ok(())
    }

    pub async fn run(mut self, iteration: u64) -> Result<Self, eyre::Error> {
        if !self.initialized {
            self.initialize().await?;
        }

        //let mut range_ending = self.next_operation + iteration;

        //if range_ending > self.raw_data.len() as u64 {
        let mut range_ending = self.raw_data.len() as u64;
        //}

        for slot in &mut self.slots {
            if *slot != SlotStatus::EMPTY && *slot != SlotStatus::ACTIVE {
                *slot = SlotStatus::ACTIVE;
            }
        }
        let mut exit_loop = false;
        for operation_number in self.next_operation..range_ending {
            let operation = &self.raw_data[operation_number as usize];

            if self.next_slot_status != SlotStatus::EMPTY {
                let memory = operation.memory.as_ref().unwrap();
                let mut new_slots = 0;

                if memory.len() as u64 > self.indexed_slots_count {
                    new_slots = memory.len() as u64 - self.indexed_slots_count
                }

                for _ in 0..new_slots {
                    self.slots[self.indexed_slots_count as usize] = self.next_slot_status;
                    self.indexed_slots_count += 1;
                }

                if operation_number - self.next_operation + 1 >= iteration {
                    self.next_operation = operation_number + 1;
                    if operation_number != 0 {
                        self.operation_codes.push(Operations::from_text(
                            self.raw_data[(operation_number - 1) as usize].op.as_str(),
                        )?)
                    }
                    exit_loop = true;
                }
            }
            match operation.op.as_str() {
                "MSTORE" => self.next_slot_status = SlotStatus::WRITING,
                "MSTORE8" => self.next_slot_status = SlotStatus::WRITING,
                "CALLDATACOPY" => self.next_slot_status = SlotStatus::WRITING,
                "RETURNDATACOPY" => self.next_slot_status = SlotStatus::WRITING,
                _ => self.next_slot_status = SlotStatus::EMPTY,
            }
            if exit_loop {
                break;
            }
        }

        Ok(self)
    }
}

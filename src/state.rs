use std::{collections::HashMap, thread::sleep, time::Duration};

use alloy::{
    primitives::{TxHash, Uint, U256},
    providers::Provider,
    rpc::types::{
        eth::Transaction,
        trace::{
            self,
            geth::{GethDebugTracingOptions, GethDefaultTracingOptions, GethTrace, StructLog},
        },
    },
};
use color_eyre::eyre;

use crate::provider;

#[derive(Debug, Clone)]
pub struct AppState {
    pub slots: Vec<SlotStatus>,
    pub slot_indexes_to_change_status: Vec<i64>,
    pub indexed_slots_count: u64,
    pub next_operation: u64,
    pub next_slot_status: SlotStatus,
    pub operation_codes: Vec<Operations>,
    pub raw_data: Vec<StructLog>,
    pub initialized: bool,
    pub transaction: Transaction,
    pub transaction_sucess: bool,
    pub history_vertical_scroll: u16,
    pub table_beginning_index: u64,
    pub operation_to_render: Option<OperationData>,
    pub read_dataset: Vec<(f64, f64)>,
    pub write_dataset: Vec<(f64, f64)>,
}

#[derive(Debug, Clone)]
pub struct OperationData {
    pub operation: Operations,
    pub params: HashMap<String, String>,
    pub remaining_gas: u64,
    pub gas_cost: u64,
    pub pc: u64,
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
            transaction_sucess: false,
            slot_indexes_to_change_status: vec![],
            history_vertical_scroll: 0,
            table_beginning_index: 0,
            operation_to_render: None,
            read_dataset: vec![],
            write_dataset: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operations {
    MSTORE,
    MSTORE8,
    MLOAD,
    CALLDATACOPY,
    MSIZE,
    EXTCODECOPY,
    CODECOPY,
    RETURNDATACOPY,
    MCOPY,
    OTHER(String),
}

impl Operations {
    pub fn text(&self) -> &str {
        match self {
            Operations::MSTORE => "MSTORE",
            Operations::MSTORE8 => "MSTORE8",
            Operations::MLOAD => "MLOAD",
            Operations::MCOPY => "MCOPY",
            Operations::CALLDATACOPY => "CALLDATACOPY",
            Operations::MSIZE => "MSIZE",
            Operations::EXTCODECOPY => "EXTCODECOPY",
            Operations::CODECOPY => "CODECOPY",
            Operations::RETURNDATACOPY => "RETURNDATACOPY",
            Operations::OTHER(op) => op.as_str(),
        }
    }
    pub fn from_text(op: &str) -> Self {
        match op {
            "MSTORE" => Operations::MSTORE,
            "MSTORE8" => Operations::MSTORE8,
            "MLOAD" => Operations::MLOAD,
            "MCOPY" => Operations::MCOPY,
            "CALLDATACOPY" => Operations::CALLDATACOPY,
            "MSIZE" => Operations::MSIZE,
            "EXTCODECOPY" => Operations::EXTCODECOPY,
            "CODECOPY" => Operations::CODECOPY,
            "RETURNDATACOPY" => Operations::RETURNDATACOPY,
            _ => Operations::OTHER(op.to_string()),
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

    pub fn from_opcode(op: &Operations) -> SlotStatus {
        match op {
            Operations::CALLDATACOPY => SlotStatus::WRITING,
            Operations::MSTORE => SlotStatus::WRITING,
            Operations::MSTORE8 => SlotStatus::WRITING,
            Operations::MLOAD => SlotStatus::READING,
            Operations::MSIZE => SlotStatus::READING,
            Operations::EXTCODECOPY => SlotStatus::WRITING,
            Operations::CODECOPY => SlotStatus::WRITING,
            Operations::RETURNDATACOPY => SlotStatus::WRITING,
            Operations::MCOPY => SlotStatus::WRITING,
            Operations::OTHER(_) => SlotStatus::EMPTY,
        }
    }
}

impl AppState {
    async fn initialize(&mut self, transaction: TxHash) -> Result<(), eyre::Error> {
        let provider = provider::HTTPProvider::new().await?;
        let transaction_result = provider.get_transaction_by_hash(transaction).await?;
        self.transaction = transaction_result;
        let opts = GethDebugTracingOptions {
            config: GethDefaultTracingOptions {
                enable_memory: Some(true),
                disable_memory: None,
                disable_stack: Some(false),
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

        let result = provider.debug_trace_transaction(transaction, opts).await?;

        match result {
            GethTrace::JS(context) => {
                std::fs::write("result3.json", context.to_string())
                    .expect("Failed to write to file");
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

    fn go_back(mut self, iteration: u64) -> Result<Self, eyre::Error> {
        // go back one iteration
        // determin the operation to index
        // determin the slot status by checking the previous operation that interacted with the memory
        let to_index = self.next_operation - iteration;
        let last_memory_affecting_op_index = self.raw_data[..to_index as usize]
            .iter()
            .rev()
            .enumerate()
            .find_map(
                |(index, operation)| match Operations::from_text(operation.op.as_str()) {
                    Operations::OTHER(_) => None,
                    _ => Some(index),
                },
            )
            .map(|index| to_index as usize - index - 1);

        if last_memory_affecting_op_index.is_none() {
            return Ok(self);
        } else {
            let operation = &self.raw_data[last_memory_affecting_op_index.unwrap()];
            self.next_slot_status =
                SlotStatus::from_opcode(&Operations::from_text(operation.op.as_str()));
            self.next_operation = last_memory_affecting_op_index.unwrap() as u64 + 2;
            for (index, slot) in self.slots.iter_mut().enumerate() {
                if index
                    >= self.raw_data[last_memory_affecting_op_index.unwrap() + 1]
                        .memory
                        .as_deref()
                        .unwrap()
                        .len()
                {
                    *slot = SlotStatus::EMPTY;
                } else if index
                    < self.raw_data[last_memory_affecting_op_index.unwrap() + 1]
                        .memory
                        .as_deref()
                        .unwrap()
                        .len()
                    && index >= operation.memory.as_deref().unwrap().len()
                {
                    *slot = self.next_slot_status;
                }
            }
            self.operation_codes.pop();
            return Ok(self);
        }
    }

    fn go_forward(mut self, iteration: u64) -> Result<Self, eyre::Error> {
        let range_ending = self.raw_data.len() as u64;

        for slot in &mut self.slots {
            if *slot != SlotStatus::EMPTY && *slot != SlotStatus::ACTIVE {
                *slot = SlotStatus::ACTIVE;
            }
        }
        let mut exit_loop = false;

        for operation_number in self.next_operation..range_ending {
            // going through all opcodes
            let operation = self.raw_data[operation_number as usize].clone();
            if self.next_slot_status != SlotStatus::EMPTY
                && self.next_slot_status != SlotStatus::INIT
            {
                // Condition to check if the memory is affected in this operation as a result of the previous operation
                // Memory is affected
                let memory = operation.memory.as_ref().unwrap();
                let mut new_slots = 0;

                // handle the new slots
                if memory.len() as u64 > self.indexed_slots_count {
                    new_slots = memory.len() as u64 - self.indexed_slots_count
                }

                for _ in 0..new_slots {
                    // New slots can only be write operations
                    self.slots[self.indexed_slots_count as usize] = self.next_slot_status;
                    self.indexed_slots_count += 1;
                }

                match self.next_slot_status {
                    SlotStatus::READING => {
                        let new_number = self.read_dataset.last().unwrap_or(&(0.0, 0.0)).1
                            + new_slots as f64
                            + self.slot_indexes_to_change_status.len() as f64;
                        self.read_dataset
                            .push((operation_number as f64, new_number));

                        if new_slots > 0 {
                            self.write_dataset.push((
                                operation_number as f64,
                                self.read_dataset.last().unwrap_or(&(0.0, 0.0)).1
                                    + new_slots as f64,
                            ));
                        } else {
                            let last_write = match self.write_dataset.last() {
                                Some(&value) => (operation_number as f64, value.1),
                                None => {
                                    // Handle the case when the vector is empty
                                    // For example, you might want to return an error or use a default value.
                                    // Here, we'll just use a default value of 0.
                                    (operation_number as f64, 0.0)
                                }
                            };
                            self.write_dataset.push(last_write);
                        }
                    }
                    SlotStatus::WRITING => {
                        let new_number = self.write_dataset.last().unwrap_or(&(0.0, 0.0)).1
                            + new_slots as f64
                            + self.slot_indexes_to_change_status.len() as f64;
                        self.write_dataset
                            .push((operation_number as f64, new_number));
                        let last_read = match self.read_dataset.last() {
                            Some(&value) => (operation_number as f64, value.1),
                            None => {
                                // Handle the case when the vector is empty
                                // For example, you might want to return an error or use a default value.
                                // Here, we'll just use a default value of 0.
                                (operation_number as f64, 0.0)
                            }
                        };
                        self.read_dataset.push(last_read);
                    }
                    _ => {}
                }

                // handle slots that need to have their status changed but aren't new
                for index in self.slot_indexes_to_change_status.clone() {
                    if index < 0 {
                        // It's a read from MCOPY
                        self.slots[(index * -1) as usize] = SlotStatus::READING;
                        continue;
                    }
                    self.slots[index as usize] = self.next_slot_status;
                }

                self.slot_indexes_to_change_status = vec![];

                // push opcode to history
                if operation_number != 0 {
                    self.operation_codes.push(Operations::from_text(
                        self.raw_data[(operation_number - 1) as usize].op.as_str(),
                    ))
                }
            } else {
                let last_write = match self.write_dataset.last() {
                    Some(&value) => (operation_number as f64, value.1),
                    None => {
                        // Handle the case when the vector is empty
                        // For example, you might want to return an error or use a default value.
                        // Here, we'll just use a default value of 0.
                        (operation_number as f64, 0.0)
                    }
                };
                let last_read = match self.read_dataset.last() {
                    Some(&value) => (operation_number as f64, value.1),
                    None => {
                        // Handle the case when the vector is empty
                        // For example, you might want to return an error or use a default value.
                        // Here, we'll just use a default value of 0.
                        (operation_number as f64, 0.0)
                    }
                };

                self.write_dataset.push(last_write);
                self.read_dataset.push(last_read);
            }

            // exit if it's the last iter
            if operation_number - self.next_operation + 1 >= iteration {
                self.next_operation = operation_number + 1;
                exit_loop = true;
            }

            match Operations::from_text(operation.op.as_str()) {
                Operations::OTHER(op) => {
                    let operation_text = Operations::from_text(&op); // Reuse the result
                    self.operation_to_render = Some(OperationData {
                        operation: operation_text.clone(), // Clone to avoid moving
                        remaining_gas: operation.gas,
                        gas_cost: operation.gas_cost,
                        pc: operation.pc,
                        params: HashMap::new(),
                    });
                    self.next_slot_status = SlotStatus::EMPTY;
                }
                _ => {
                    let operation_text = Operations::from_text(operation.op.as_str()); // Only one call
                    let params = self.handle_opcode(operation_text.clone(), operation.stack); // Clone for reuse
                    self.operation_to_render = Some(OperationData {
                        operation: operation_text,
                        remaining_gas: operation.gas,
                        gas_cost: operation.gas_cost,
                        pc: operation.pc,
                        params,
                    });
                }
            }
            

            if exit_loop {
                break;
            }
        }

        Ok(self)
    }

    fn handle_opcode(
        &mut self,
        opcode: Operations,
        stack: Option<Vec<U256>>,
    ) -> HashMap<String, String> {
        self.next_slot_status = SlotStatus::from_opcode(&opcode);
        let mut params: HashMap<String, String> = HashMap::new();
        // handle other changes that are applied to the already existing slots
        match opcode {
            Operations::MCOPY => {
                let unwrapped_stack = stack.as_ref().unwrap();
                let memory_offset =
                    unwrapped_stack.get(unwrapped_stack.len() - 1).unwrap() / Uint::from(32);
                let copy_offset =
                    unwrapped_stack.get(unwrapped_stack.len() - 2).unwrap() / Uint::from(32);
                let memory_end = ((unwrapped_stack.get(unwrapped_stack.len() - 3).unwrap())
                    + Uint::from(31))
                    / Uint::from(32);
                for i in memory_offset.to::<i64>()
                    ..=(memory_offset + memory_end - Uint::from(1)).to::<i64>()
                {
                    self.slot_indexes_to_change_status.push(i);
                }
                for i in
                    copy_offset.to::<i64>()..=(copy_offset + memory_end - Uint::from(1)).to::<i64>()
                {
                    self.slot_indexes_to_change_status.push(i * -1);
                }
                params.insert(
                    "Destination Offset".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 1)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
                params.insert(
                    "Source Offset".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 2)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
                params.insert(
                    "Byte Size".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 3)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
            }
            Operations::MSTORE => {
                let unwrapped_stack = stack.as_ref().unwrap();
                let memory_offset = unwrapped_stack.last().unwrap() / Uint::from(32);
                self.slot_indexes_to_change_status = vec![memory_offset.to::<i64>()];
                params.insert(
                    "Destination Offset".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 1)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
                params.insert(
                    "Value".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 2)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
            }
            Operations::EXTCODECOPY => {
                let unwrapped_stack = stack.as_ref().unwrap();
                let memory_offset =
                    unwrapped_stack.get(unwrapped_stack.len() - 2).unwrap() / Uint::from(32);
                let memory_end = (unwrapped_stack.get(unwrapped_stack.len() - 4).unwrap()
                    + Uint::from(31))
                    / Uint::from(32);
                for i in memory_offset.to::<i64>()
                    ..=(memory_offset + memory_end - Uint::from(1)).to::<i64>()
                {
                    self.slot_indexes_to_change_status.push(i);
                }
                //params.insert("Address".to_string(), unwrapped_stack.get(unwrapped_stack.len() - 1).unwrap().to_string());
                params.insert(
                    "Destination Offset".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 2)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
                params.insert(
                    "Source Offset".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 3)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
                params.insert(
                    "Byte Size".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 4)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
            }
            Operations::CODECOPY => {
                let unwrapped_stack = stack.as_ref().unwrap();
                let memory_offset =
                    unwrapped_stack.get(unwrapped_stack.len() - 1).unwrap() / Uint::from(32);
                let memory_end = ((unwrapped_stack.get(unwrapped_stack.len() - 3).unwrap())
                    + Uint::from(31))
                    / Uint::from(32);
                for i in memory_offset.to::<i64>()
                    ..=(memory_offset + memory_end - Uint::from(1)).to::<i64>()
                {
                    self.slot_indexes_to_change_status.push(i);
                }
                params.insert(
                    "Destination Offset".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 1)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
                params.insert(
                    "Source Offset".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 2)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
                params.insert(
                    "Byte Size".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 3)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
            }
            Operations::RETURNDATACOPY => {
                let unwrapped_stack = stack.as_ref().unwrap();
                let memory_offset =
                    unwrapped_stack.get(unwrapped_stack.len() - 1).unwrap() / Uint::from(32);
                let memory_end = ((unwrapped_stack.get(unwrapped_stack.len() - 3).unwrap())
                    + Uint::from(31))
                    / Uint::from(32);
                for i in memory_offset.to::<i64>()
                    ..=(memory_offset + memory_end - Uint::from(1)).to::<i64>()
                {
                    self.slot_indexes_to_change_status.push(i);
                }

                params.insert(
                    "Destination Offset".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 1)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
                params.insert(
                    "Source Offset".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 2)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
                params.insert(
                    "Byte Size".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 3)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
            }
            Operations::MSTORE8 => {
                // Writes one byte value
                // TODO: Check what happens if the offset is at the end of a slot and the value needs more space, does it overflow to the next slot?
                let unwrapped_stack: &Vec<Uint<256, 4>> = stack.as_ref().unwrap();
                let memory_offset = unwrapped_stack.last().unwrap() / Uint::from(32);
                self.slot_indexes_to_change_status = vec![memory_offset.saturating_to::<i64>()];
                params.insert(
                    "Destination Offset".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 1)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );

                params.insert(
                    "Value".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 2)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
            }
            Operations::MLOAD => {
                let unwrapped_stack: &Vec<Uint<256, 4>> = stack.as_ref().unwrap();
                let memory_offset = unwrapped_stack.last().unwrap() / Uint::from(32);
                self.slot_indexes_to_change_status = vec![memory_offset.to::<i64>()];
                params.insert(
                    "Source Offset".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 1)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
            }
            Operations::CALLDATACOPY => {
                let unwrapped_stack = stack.as_ref().unwrap();
                let memory_offset =
                    unwrapped_stack.get(unwrapped_stack.len() - 1).unwrap() / Uint::from(32);
                let memory_end = ((unwrapped_stack.get(unwrapped_stack.len() - 3).unwrap())
                    + Uint::from(31))
                    / Uint::from(32);
                for i in memory_offset.to::<i64>()
                    ..=(memory_offset + memory_end - Uint::from(1)).to::<i64>()
                {
                    self.slot_indexes_to_change_status.push(i);
                }

                params.insert(
                    "Destination Offset".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 1)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
                params.insert(
                    "Source Offset".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 2)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
                params.insert(
                    "Byte Size".to_string(),
                    unwrapped_stack
                        .get(unwrapped_stack.len() - 3)
                        .unwrap()
                        .to::<u64>()
                        .to_string(),
                );
            }
            Operations::MSIZE => {
                self.slot_indexes_to_change_status =
                    (0..self.indexed_slots_count).map(|x| x as i64).collect();
            }
            Operations::OTHER(_) => {}
        };
        params
    }

    pub async fn run(
        mut self,
        transaction: TxHash,
        iteration: u64,
        forward: bool,
        pause: bool,
    ) -> Result<Self, eyre::Error> {
        if !self.initialized {
            self.initialize(transaction).await?;
        }
        if pause {
            return Ok(self);
        }
        if !forward {
            return self.go_back(iteration);
        }

        self.go_forward(iteration)
    }
}

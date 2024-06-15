use std::collections::HashMap;

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
use opcode_parser::Operations;

use crate::provider;

#[derive(Debug, Clone)]
pub struct AppState {
    /// App mode
    pub mode: AppMode,
    /// States for the transactions
    pub transaction_states: Vec<TransactionState>,
}

impl Default for AppState {
    fn default() -> Self {
        Self { mode: AppMode::NORMAL, transaction_states: vec![TransactionState::default()] }
    }
}

impl AppState {
    pub async fn init(
        &mut self,
        rpc: &str,
        transactions: Vec<TxHash>,
    ) -> Result<&mut Self, eyre::Error> {
        let mut first_transaction_state = TransactionState::default();
        first_transaction_state.initialize(transactions[0], rpc).await.unwrap();

        let mut transaction_states = vec![first_transaction_state];

        if transactions.len() > 1 {
            // versus view
            self.mode = AppMode::VERSUS;
            let mut second_transaction_state = TransactionState::default();
            second_transaction_state.initialize(transactions[1], rpc).await.unwrap();
            transaction_states.push(second_transaction_state);
        }

        self.transaction_states = transaction_states;

        Ok(self)
    }
}

#[derive(Debug, Clone)]
pub enum AppMode {
    VERSUS,
    NORMAL,
}

#[derive(Debug, Clone, Default)]
pub struct TransactionState {
    /// Vector of slots with values of SlotStatus
    pub slots: Vec<SlotStatus>,
    /// Slots that are not new and need to change their status in the next run
    pub slot_indexes_to_change_status: Vec<i64>,
    /// Number of indexed slots
    pub indexed_slots_count: u64,
    /// Next operation number to process
    pub next_operation: u64,
    /// Next slots status
    pub next_slot_status: SlotStatus,
    /// History of opcodes
    pub operation_codes: Vec<Operations>,
    /// Raw returned data by the trace transaction call
    pub raw_data: Vec<StructLog>,
    /// Transaction details
    pub transaction: Transaction,
    /// Success of the transaction
    pub transaction_success: bool,
    /// Position of the scroller in the history box
    pub history_vertical_scroll: u16,
    /// The slot number to display at the top for raw mode and the line to display at the top for
    /// normal mode
    pub table_beginning_index: u64,
    /// Operation data to render in the operation info box
    pub operation_to_render: OperationData,
    /// The read operations chart dataset
    pub read_dataset: Vec<(f64, f64)>,
    /// The write operations chart dataset
    pub write_dataset: Vec<(f64, f64)>,
    /// Display help box
    pub help: bool,
    /// Display raw memory data
    pub display_memory_data: bool,
    /// Pause the process
    pub pause: bool,
}

#[derive(Debug, Clone)]
pub struct OperationData {
    pub operation: Operations,
    pub params: HashMap<String, String>,
    pub remaining_gas: u64,
    pub gas_cost: u64,
    pub pc: u64,
}

impl Default for OperationData {
    fn default() -> Self {
        Self {
            operation: Operations::OTHER("".to_string()),
            params: HashMap::new(),
            remaining_gas: 0,
            gas_cost: 0,
            pc: 0,
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
    UNREAD,
}

impl Default for SlotStatus {
    fn default() -> Self {
        Self::INIT
    }
}

impl SlotStatus {
    pub fn text(&self) -> &'static str {
        match self {
            SlotStatus::INIT => "Initializing",
            SlotStatus::EMPTY => "Empty",
            SlotStatus::ACTIVE => "Active",
            SlotStatus::READING => "Reading",
            SlotStatus::WRITING => "Writing",
            SlotStatus::UNREAD => "Unread",
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

impl TransactionState {
    pub async fn initialize(&mut self, transaction: TxHash, rpc: &str) -> Result<(), eyre::Error> {
        let provider = provider::HTTPProvider::new(rpc).await?;
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
                self.transaction_success = !serde_json::from_value(context["failed"].clone())?;
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

        Ok(())
    }

    fn go_back(&mut self, iteration: u64) -> Result<&mut Self, eyre::Error> {
        // go back one iteration
        // determin the operation to index
        // determin the slot status by checking the previous operation that interacted with the
        // memory
        let to_index = self.next_operation - iteration;
        let last_memory_affecting_op_index = self.raw_data[..to_index as usize]
            .iter()
            .rev()
            .enumerate()
            .find_map(|(index, operation)| match Operations::from_text(operation.op.as_str()) {
                Operations::OTHER(_) => None,
                _ => Some(index),
            })
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

    fn go_forward(&mut self, iteration: u64) -> Result<&mut Self, eyre::Error> {
        let range_ending = self.raw_data.len() as u64;

        for slot in &mut self.slots {
            if *slot == SlotStatus::INIT || *slot == SlotStatus::READING {
                *slot = SlotStatus::ACTIVE;
            }
            if *slot == SlotStatus::WRITING {
                *slot = SlotStatus::UNREAD;
            }
        }

        for operation_number in self.next_operation..range_ending {
            // going through all opcodes
            let operation = self.raw_data[operation_number as usize].clone();
            if self.next_slot_status != SlotStatus::EMPTY
                && self.next_slot_status != SlotStatus::INIT
            {
                // Condition to check if the memory is affected in this operation as a result of the
                // previous operation Memory is affected
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
                        self.read_dataset.push((operation_number as f64, new_number));

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
                                    // For example, you might want to return an error or use a
                                    // default value.
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
                        self.write_dataset.push((operation_number as f64, new_number));
                        let last_read = match self.read_dataset.last() {
                            Some(&value) => (operation_number as f64, value.1),
                            None => {
                                // Handle the case when the vector is empty
                                // For example, you might want to return an error or use a default
                                // value. Here, we'll just use a
                                // default value of 0.
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

            // push opcode to history
            if operation_number != 0 {
                self.operation_codes.push(Operations::from_text(
                    self.raw_data[(operation_number - 1) as usize].op.as_str(),
                ));
            }

            match Operations::from_text(operation.op.as_str()) {
                Operations::OTHER(op) => {
                    let operation_text = Operations::from_text(&op); // Reuse the result
                    self.operation_to_render = OperationData {
                        operation: operation_text.clone(), // Clone to avoid moving
                        remaining_gas: operation.gas,
                        gas_cost: operation.gas_cost,
                        pc: operation.pc,
                        params: HashMap::new(),
                    };
                    self.next_slot_status = SlotStatus::EMPTY;
                }
                _ => {
                    let operation_text = Operations::from_text(operation.op.as_str()); // Only one call
                    self.handle_opcode(operation_text.clone(), operation.stack.clone());
                    self.operation_to_render = OperationData {
                        operation: operation_text.clone(),
                        remaining_gas: operation.gas,
                        gas_cost: operation.gas_cost,
                        pc: operation.pc,
                        params: operation_text.parse_args(operation.stack),
                    };
                }
            }

            // exit if it's the last iter
            if operation_number - self.next_operation + 1 >= iteration {
                self.next_operation = operation_number + 1;
                break;
            }
        }

        Ok(self)
    }

    fn handle_opcode(&mut self, opcode: Operations, stack: Option<Vec<U256>>) {
        self.next_slot_status = SlotStatus::from_opcode(&opcode);
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
            }
            Operations::MSTORE => {
                let unwrapped_stack = stack.as_ref().unwrap();
                let memory_offset = unwrapped_stack.last().unwrap() / Uint::from(32);
                self.slot_indexes_to_change_status = vec![memory_offset.to::<i64>()];
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
            }
            Operations::MSTORE8 => {
                // Writes one byte value
                // TODO: Check what happens if the offset is at the end of a slot and the value
                // needs more space, does it overflow to the next slot?
                let unwrapped_stack: &Vec<Uint<256, 4>> = stack.as_ref().unwrap();
                let memory_offset = unwrapped_stack.last().unwrap() / Uint::from(32);
                self.slot_indexes_to_change_status = vec![memory_offset.saturating_to::<i64>()];
            }
            Operations::MLOAD => {
                let unwrapped_stack: &Vec<Uint<256, 4>> = stack.as_ref().unwrap();
                let memory_offset = unwrapped_stack.last().unwrap() / Uint::from(32);
                self.slot_indexes_to_change_status = vec![memory_offset.to::<i64>()];
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
            }
            Operations::MSIZE => {
                self.slot_indexes_to_change_status =
                    (0..self.indexed_slots_count).map(|x| x as i64).collect();
            }
            Operations::OTHER(_) => {}
        };
    }

    pub async fn run(
        &mut self,
        iteration: u64,
        forward: bool,
        pause: bool,
    ) -> Result<&mut Self, eyre::Error> {
        self.pause = pause;
        if pause {
            return Ok(self);
        }
        if !forward {
            return self.go_back(iteration);
        }

        self.go_forward(iteration)
    }
}

use alloy::{
    primitives::{fixed_bytes, Uint, U256},
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
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
}

impl Operations {
    pub fn text(&self) -> &'static str {
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
        }
    }
    pub fn from_text(op: &str) -> Result<Self, ()> {
        match op {
            "MSTORE" => return Ok(Operations::MSTORE),
            "MSTORE8" => return Ok(Operations::MSTORE8),
            "MLOAD" => return Ok(Operations::MLOAD),
            "MCOPY" => return Ok(Operations::MCOPY),
            "CALLDATACOPY" => return Ok(Operations::CALLDATACOPY),
            "MSIZE" => return Ok(Operations::MSIZE),
            "EXTCODECOPY" => return Ok(Operations::EXTCODECOPY),
            "CODECOPY" => return Ok(Operations::CODECOPY),
            "RETURNDATACOPY" => return Ok(Operations::RETURNDATACOPY),
            _ => return Err(()),
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

    pub fn from_opcode(op: Operations) -> SlotStatus {
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
        }
    }
}

impl AppState {
    async fn initialize(&mut self) -> Result<(), eyre::Error> {
        let provider = provider::HTTPProvider::new().await?;
        let tx_hash =
            fixed_bytes!("970bf06f06ee47ce411f357b73b07a5267c72b838eb11950399144baa05e8740");
        let transaction_result = provider.get_transaction_by_hash(tx_hash).await?;
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

        let result = provider.debug_trace_transaction(tx_hash, opts).await?;

        match result {
            GethTrace::JS(context) => {
                // std::fs::write("result3.json", context.to_string())
                //     .expect("Failed to write to file");
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
            .find_map(|(index, operation)| {
                if Operations::from_text(operation.op.as_str()).is_ok() {
                    Some(index)
                } else {
                    None
                }
            })
            .map(|index| to_index as usize - index - 1);

        if last_memory_affecting_op_index.is_none() {
            return Ok(self);
        } else {
            let operation = &self.raw_data[last_memory_affecting_op_index.unwrap()];
            self.next_slot_status =
                SlotStatus::from_opcode(Operations::from_text(operation.op.as_str()).unwrap());
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
            if self.next_slot_status != SlotStatus::EMPTY {
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
                    self.operation_codes.push(
                        Operations::from_text(
                            self.raw_data[(operation_number - 1) as usize].op.as_str(),
                        )
                        .unwrap(),
                    )
                }

                // exit if it's the last iter
                if operation_number - self.next_operation + 1 >= iteration {
                    self.next_operation = operation_number + 1;
                    exit_loop = true;
                }
            }

            if let Ok(opcode) = Operations::from_text(operation.op.as_str()) {
                self.handle_opcode(opcode, operation.stack);
            } else {
                self.next_slot_status = SlotStatus::EMPTY;
            }

            if exit_loop {
                break;
            }
        }

        Ok(self)
    }

    fn handle_opcode(&mut self, opcode: Operations, stack: Option<Vec<U256>>) {
        self.next_slot_status = SlotStatus::from_opcode(opcode);
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
                let memory_offset = stack.as_ref().unwrap().last().unwrap() / Uint::from(32);
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
                // TODO: Check what happens if the offset is at the end of a slot and the value needs more space, does it overflow to the next slot?
                let memory_offset = stack.as_ref().unwrap().last().unwrap() / Uint::from(32);
                self.slot_indexes_to_change_status = vec![memory_offset.saturating_to::<i64>()];
            }
            Operations::MLOAD => {
                let memory_offset = stack.as_ref().unwrap().last().unwrap() / Uint::from(32);
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
        }
    }

    pub async fn run(
        mut self,
        iteration: u64,
        forward: bool,
        pause: bool,
    ) -> Result<Self, eyre::Error> {
        if !self.initialized {
            self.initialize().await?;
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

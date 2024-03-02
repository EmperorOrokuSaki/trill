use std::io;

use crate::{
    provider,
    tui::{self, Event},
};
use alloy::{
    primitives::fixed_bytes,
    providers::Provider,
    rpc::types::trace::{
        self,
        geth::{GethDebugTracingOptions, GethDefaultTracingOptions, GethTrace, StructLog},
    },
};
use color_eyre::eyre::{self, eyre};
use crossterm::event::KeyCode::Char;
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{
        block::{Position, Title},
        Block, Borders, Cell, Paragraph, Row, Table, TableState,
    },
};

#[derive(Debug)]
pub struct App {
    iteration: u64,
    exit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            iteration: 1,
            exit: false,
        }
    }
}

pub struct AppState {
    slots: Vec<SlotStatus>,
    indexed_slots_count: u64,
    next_operation: u64,
    next_slot_status: SlotStatus,
    next_operation_code: Option<Operations>,
    raw_data: Vec<StructLog>,
    initialized: bool,
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
            next_operation_code: None,
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
    pub fn fromText(op: &str) -> Option<Self> {
        match op {
            "MSTORE" => return Some(Operations::MSTORE),
            "MSTORE8" => return Some(Operations::MSTORE8),
            "MLOAD" => return Some(Operations::MLOAD),
            "CALLDATACOPY" => return Some(Operations::CALLDATACOPY),
            "RETURNDATACOPY" => return Some(Operations::RETURNDATACOPY),
        _ => None
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
                fixed_bytes!("cd3d9bba59cb634070a0b84bf333c97daed0eb6244929f3ba27b847365bbe546"),
                opts,
            )
            .await?;

        match result {
            GethTrace::JS(context) => {

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

    async fn run(mut self, iteration: u64) -> Result<Self, eyre::Error> {
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
                    if operation_number != 0 {self.next_operation_code = Operations::fromText(self.raw_data[(operation_number - 1) as usize].op.as_str());}
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

impl App {
    /// runs the application's main loop until the user quits
    pub async fn run(&mut self) -> color_eyre::Result<()> {
        let mut tui = tui::Tui::new()?
            .tick_rate(1.0) // 4 ticks per second
            .frame_rate(1.0); // 30 frames per second

        tui.enter()?; // Starts event handler, enters raw mode, enters alternate screen
        let mut app_state = AppState::default();
        loop {
            app_state = AppState::run(app_state, self.iteration).await?;

            tui.draw(|f| {
                // Deref allows calling tui.terminal.draw
                self.render_frame(f, &mut app_state);
            })?;

            if let Some(evt) = tui.next().await {
                // tui.next().await blocks till next event
                self.handle_event(evt)?;
            };

            if self.exit {
                break;
            }
        }

        tui.exit()?; // stops event handler, exits raw mode, exits alternate screen

        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame, mut state: &mut AppState) {
        frame.render_stateful_widget(self, frame.size(), &mut state);
    }

    fn handle_event(&mut self, event: Event) -> io::Result<()> {
        if let Event::Key(key) = event {
            match key.code {
                Char('q') => self.exit = true,
                _ => {}
            }
        }
        Ok(())
    }
}

impl StatefulWidget for &App {
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let info_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ])
            .split(layout[1]);

        let title = Title::from(" Trill ".bold());
        let instructions = Title::from(Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]));
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let mut s = TableState::default();
        let mut rows: Vec<Row> = vec![];
        let mut row: Vec<Cell> = vec![];
        for slot in 0..state.slots.len() {
            match state.slots[slot] {
                SlotStatus::EMPTY => row.push(Cell::new("■").style(Style::new().gray())),
                SlotStatus::ACTIVE => row.push(Cell::new("■").style(Style::new().green())),
                SlotStatus::READING => row.push(Cell::new("■").style(Style::new().blue())),
                SlotStatus::WRITING => row.push(Cell::new("■").style(Style::new().red())),
                SlotStatus::INIT => (),
            }
            if slot % 100 == 99 {
                rows.push(Row::new(row.clone()));
                row.clear();
            }
        }
        ratatui::widgets::StatefulWidget::render(
            Table::new(rows, [Constraint::Length(1); 100]).block(block),
            layout[0],
            buf,
            &mut s,
        );
        // TX INFO
        let title = Title::from(" Transaction info ".bold());
        let tx_info_block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK);
        let tx_info_rows = vec![
            Row::new(vec![
                Cell::new("Hash").style(Style::new().gray()),
                Cell::new("0xcd3d9bba59cb634070a0b84bf333c97daed0eb6244929f3ba27b847365bbe546")
                    .style(Style::new().gray()),
            ]),
            Row::new(vec![
                Cell::new("From").style(Style::new().gray()),
                Cell::new("0xcd3d9bba59cb634070a0b84bf333c97daed0eb6244929f3ba27b847365bbe546")
                    .style(Style::new().gray()),
            ]),
            Row::new(vec![
                Cell::new("To").style(Style::new().gray()),
                Cell::new("0xcd3d9bba59cb634070a0b84bf333c97daed0eb6244929f3ba27b847365bbe546")
                    .style(Style::new().gray()),
            ]),Row::new(vec![
                Cell::new("Block Hash").style(Style::new().gray()),
                Cell::new("0xcd3d9bba59cb634070a0b84bf333c97daed0eb6244929f3ba27b847365bbe546")
                    .style(Style::new().gray()),
            ]),Row::new(vec![
                Cell::new("Block Number").style(Style::new().gray()),
                Cell::new("18554494")
                    .style(Style::new().gray()),
            ]),
            Row::new(vec![
                Cell::new("Success").style(Style::new().gray()),
                Cell::new("true")
                    .style(Style::new().green()),
            ]),
            Row::new(vec![
                Cell::new("Gas used").style(Style::new().gray()),
                Cell::new("15556674152").style(Style::new().gray()),
            ]),
        ];
        let tx_info_table = Table::new(
            tx_info_rows,
            [Constraint::Percentage(20), Constraint::Fill(1)],
        )
        .block(tx_info_block);
        ratatui::widgets::StatefulWidget::render(tx_info_table, info_layout[0], buf, &mut s);
        // Operation INFO
        let title = Title::from(" Operation info ".bold());
        let op_info_block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK);
        let vec = vec![
            Row::new(vec![
                Cell::new("Op code").style(Style::new().gray()),
                Cell::new(state.next_operation_code.unwrap_or(Operations::MLOAD).text())
                    .style(Style::new().red()),
            ]),
            Row::new(vec![
                Cell::new("Gas cost").style(Style::new().gray()),
                Cell::new("3")
                    .style(Style::new().gray()),
            ]),
        ];
        let op_info_rows = vec;
        let op_info_table = Table::new(
            op_info_rows,
            [Constraint::Percentage(20), Constraint::Fill(1)],
        )
        .block(op_info_block);
        ratatui::widgets::StatefulWidget::render(op_info_table, info_layout[1], buf, &mut s);
    }
    type State = AppState;
}

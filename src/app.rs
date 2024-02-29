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
        Block, Borders, Cell, Row, Table, TableState,
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
            iteration: 5,
            exit: false,
        }
    }
}

pub struct AppState {
    slots: Vec<SlotStatus>,
    indexed_slots_count: u64,
    next_slot: u64,
    next_slot_status: SlotStatus,
    raw_data: Vec<StructLog>,
    initialized: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            slots: vec![],
            indexed_slots_count: 0,
            next_slot: 0,
            next_slot_status: SlotStatus::INIT,
            raw_data: vec![],
            initialized: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SlotStatus {
    INIT,
    EMPTY,
    ACTIVE,
    READING,
    WRITING,
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
                fixed_bytes!("52ac113a9ad810a0af4e23c656ea7bfbcb43b1cac933befb02a23d7f75283fc7"),
                opts,
            )
            .await?;
        match result {
            GethTrace::JS(context) => {
                std::fs::write("result1.json", context.to_string())
                    .expect("Failed to write to file");
                self.raw_data = serde_json::from_value(context["structLogs"].clone())?;
                self.slots = vec![
                    SlotStatus::EMPTY;
                    self.raw_data
                        .iter()
                        .filter(|operation| operation.memory.is_some())
                        .count()
                ];
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

        let mut range_ending = self.next_slot + iteration;

        if range_ending > self.raw_data.len() as u64 {
            range_ending = self.raw_data.len() as u64;
        }

        for operation_number in self.next_slot..range_ending {
            let operation = &self.raw_data[operation_number as usize];

            if operation.memory.is_some() {
                let memory = operation.memory.as_ref().unwrap();
                let mut new_slots = 0;
                if memory.len() as u64 > self.indexed_slots_count {
                    new_slots = memory.len() as u64 - self.indexed_slots_count
                }
                for _ in 0..new_slots {
                    self.slots[self.indexed_slots_count as usize] = self.next_slot_status;
                    self.indexed_slots_count += 1;
                }
            }
            match operation.op.as_str() {
                "MSTORE" => self.next_slot_status = SlotStatus::WRITING,
                _ => (),
            }
        }

        self.next_slot = range_ending;

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
            area,
            buf,
            &mut s,
        );
    }
    type State = AppState;
}

use std::io;

use crate::{
    provider,
    tui::{self, Event},
};
use alloy::{
    hex::FromHex,
    primitives::{address, b256, fixed_bytes, B256, U256},
    providers::Provider,
    rpc::types::trace::{
        self,
        geth::{
            CallConfig, GethDebugBuiltInTracerType, GethDebugTracerType, GethDebugTracingOptions,
            GethDefaultTracingOptions, GethTrace,
        },
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

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
}

pub struct AppState {
    data: Vec<U256>,
}

impl AppState {
    async fn run() -> Result<AppState, eyre::Error> {
        let data: Vec<U256> = vec![];
        println!("HE");
        let provider = provider::HTTPProvider::new().await?;
        let opts = GethDebugTracingOptions {
            config: GethDefaultTracingOptions {
                enable_memory: Some(true),
                disable_memory: None,
                disable_stack: Some(true),
                disable_storage: Some(true),
                enable_return_data: Some(false),
                disable_return_data: None,
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
                let result_json =
                    serde_json::to_string(&context).expect("Failed to serialize result to JSON");

                // Write the JSON string to a file named "result.json"
                std::fs::write("result.json", result_json).expect("Failed to write to file");
            }
            _ => (),
        }
        Ok(Self { data: data })
    }
}

impl App {
    /// runs the application's main loop until the user quits
    pub async fn run(&mut self) -> color_eyre::Result<()> {
        let mut tui = tui::Tui::new()?
            .tick_rate(1.0) // 4 ticks per second
            .frame_rate(1.0); // 30 frames per second

        tui.enter()?; // Starts event handler, enters raw mode, enters alternate screen
        let mut state = AppState::run().await?;
        loop {
            tui.draw(|f| {
                // Deref allows calling `tui.terminal.draw`
                self.render_frame(f, &mut state);
            })?;

            if let Some(evt) = tui.next().await {
                // `tui.next().await` blocks till next event
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
        for slot in 0..1000 {
            if state.data[slot] == U256::from(0) {
                row.push(Cell::new("■").style(Style::new().blue()));
            } else {
                row.push(Cell::new("■").style(Style::new().red()));
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

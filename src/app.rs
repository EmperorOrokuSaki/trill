use std::io;

use alloy::{primitives::{address, U256}, providers::{Provider, ProviderBuilder}};
use color_eyre::eyre::{self, eyre};
use crossterm::event::{
    KeyCode::{ Char},
};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{
        block::{Position, Title},
        Block, Borders, Cell, Row, Table, TableState,
    },
};
use crate::{provider, tui::{self, Event}};

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
}

pub struct AppState {
    data: Vec<U256>
}

impl AppState {
    async fn run() -> Result<AppState, eyre::Error> {
        let mut data : Vec<U256> = vec![];
        let provider = provider::HTTPProvider::new().await?;
        let pool_address = address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
        let method_name = std::borrow::Cow::from("eth_call");
        // let batch = provider.raw_request(method_name, serde_json::json!([
        //     {
        //         "to": "0xebe8efa441b9302a0d7eaecc277c09d20d684540",
        //         "data": "0x0be5b6ba"
        //     },
        //     "latest",
        //     {
        //         "0xebe8efa441b9302a0d7eaecc277c09d20d684540": {
        //             "code": "0x6080604052348015600f57600080fd5b506004361060285760003560e01c80630be5b6ba14602d575b600080fd5b60336045565b60408051918252519081900360200190f35b6007549056fea265627a7a723058206f26bd0433456354d8d1228d8fe524678a8aeeb0594851395bdbd35efc2a65f164736f6c634300050a0032"
        //         }
        //     }
        // ])).await?;
        for slot in 0..1000 {
            let storage_slot = U256::from(slot);
            let storage = provider
                .get_storage_at(pool_address, storage_slot, None)
                .await?;
            data.push(storage);
        }
        Ok(Self {
            data: data
        })
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
        let mut rows : Vec<Row> = vec![];
        let mut row : Vec<Cell> = vec![];
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

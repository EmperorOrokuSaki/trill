use std::io;

use alloy::{primitives::{address, U256}, providers::{Provider, ProviderBuilder}};
use color_eyre::eyre::eyre;
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

use crate::tui::{self, Event};

#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    exit: bool,
}

impl App {
    /// runs the application's main loop until the user quits
    pub async fn run(&mut self) -> color_eyre::Result<()> {
        let mut tui = tui::Tui::new()?
            .tick_rate(4.0) // 4 ticks per second
            .frame_rate(30.0); // 30 frames per second

        tui.enter()?; // Starts event handler, enters raw mode, enters alternate screen

        loop {
            // Get storage slot 0 from the Uniswap V3 USDC-ETH pool on Ethereum mainnet.
            let pool_address = address!("88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640");
            let storage_slot = U256::from(0);
            let storage = provider
                .get_storage_at(pool_address, storage_slot, None)
                .await?;
    
            dbg!(storage);
 
            tui.draw(|f| {
                // Deref allows calling `tui.terminal.draw`
                self.render_frame(f);
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

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_stateful_widget(self, frame.size(), &mut 0);
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

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    fn decrement_counter(&mut self) {
        self.counter -= 1;
    }
}

impl StatefulWidget for &App {
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut i32) {
        let mut rows: Vec<Row> = Vec::new();

        for k in 1..9 {
            let mut row = Vec::<Cell>::new();
            for i in 1..101 {
                if (i + k) % 2 == 0 {
                    // even
                    row.push(Cell::new("■").style(Style::new().red()));
                } else if (i + k) % 3 == 0 {
                    row.push(Cell::new("■").style(Style::new().blue()));
                } else {
                    row.push(Cell::new("■").style(Style::new().white()));
                }
            }
            rows.push(Row::new(row));
        }

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
        ratatui::widgets::StatefulWidget::render(
            Table::new(rows, [Constraint::Length(1); 100]).block(block),
            area,
            buf,
            &mut s,
        );
    }

    type State = i32;
}

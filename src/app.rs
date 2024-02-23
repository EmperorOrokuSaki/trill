use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{prelude::*, symbols::border, widgets::{block::{Position, Title}, Block, Borders, Cell, Row, Table, TableState}};

use crate::tui;


#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    exit: bool,
}

impl App {

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_stateful_widget(self, frame.size(), &mut 0);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            _ => {}
        }
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

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);
        let mut s = TableState::default();
        ratatui::widgets::StatefulWidget::render(Table::new(rows, [Constraint::Length(1); 100]).block(block), area, buf, &mut s);
        }
    
    type State = i32;
}

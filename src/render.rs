use alloy::primitives::Address;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Borders, Cell, List, ListDirection, Row, Table, TableState,
    },
};

use crate::state::{AppState, Operations, SlotStatus};

pub struct RenderData<'a> {
    pub area: Rect,
    pub buf: &'a mut Buffer,
    pub state: &'a mut AppState,
}

impl<'a> RenderData<'a> {
    fn render_memory(&mut self, layout: Rect) {
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
        for slot in 0..self.state.slots.len() {
            match self.state.slots[slot] {
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
            layout,
            self.buf,
            &mut s,
        );
    }

    fn render_transaction_box(&mut self, layout: Rect) {
        let transaction = &self.state.transaction;
        let title = Title::from(" Transaction info ".bold());
        let tx_info_block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK);
        let tx_info_rows = vec![
            Row::new(vec![
                Cell::new("Hash").style(Style::new().gray().bold()),
                Cell::new(transaction.hash.to_string()).style(Style::new().gray()),
            ]),
            Row::new(vec![
                Cell::new("From").style(Style::new().gray().bold()),
                Cell::new(transaction.from.to_string()).style(Style::new().gray()),
            ]),
            Row::new(vec![
                Cell::new("To").style(Style::new().gray().bold()),
                Cell::new(transaction.to.unwrap().to_string()).style(Style::new().gray()),
            ]),
            Row::new(vec![
                Cell::new("Block Hash").style(Style::new().gray().bold()),
                Cell::new(transaction.block_hash.unwrap().to_string()).style(Style::new().gray()),
            ]),
            Row::new(vec![
                Cell::new("Block Number").style(Style::new().gray().bold()),
                Cell::new(transaction.block_number.unwrap().to_string()).style(Style::new().gray()),
            ]),
            Row::new(vec![
                Cell::new("Success").style(Style::new().gray().bold()),
                Cell::new(self.state.transaction_sucess.to_string()).style(Style::new().green()),
            ]),
            Row::new(vec![
                Cell::new("Gas used").style(Style::new().gray().bold()),
                Cell::new(transaction.gas.to_string()).style(Style::new().gray()),
            ]),
        ];
        let tx_info_table = Table::new(
            tx_info_rows,
            [Constraint::Percentage(20), Constraint::Fill(1)],
        )
        .block(tx_info_block);
        let mut s = TableState::default();

        ratatui::widgets::StatefulWidget::render(tx_info_table, layout, self.buf, &mut s);
    }

    fn render_current_operation_box(&mut self, layout: Rect) {
        let title = Title::from(" Operation info ".bold());
        let op_info_block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK);
        let vec = vec![
            Row::new(vec![
                Cell::new("Op code").style(Style::new().gray()),
                Cell::new(
                    self.state
                        .operation_codes
                        .last()
                        .unwrap_or(&Operations::MLOAD)
                        .text(),
                )
                .style(Style::new().red()),
            ]),
            Row::new(vec![
                Cell::new("Gas cost").style(Style::new().gray()),
                Cell::new("3").style(Style::new().gray()),
            ]),
        ];
        let op_info_rows = vec;
        let op_info_table = Table::new(
            op_info_rows,
            [Constraint::Percentage(20), Constraint::Fill(1)],
        )
        .block(op_info_block);
        let mut s = TableState::default();

        ratatui::widgets::StatefulWidget::render(op_info_table, layout, self.buf, &mut s);
    }

    fn render_operation_history(&mut self, layout: Rect) {
        let title = Title::from(" Operation history ".bold());
        let history_info_block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK);
        let list = List::new(
            self.state
                .operation_codes
                .iter()
                .map(|op| Text::styled(op.text(), Style::default().green())),
        )
        .block(history_info_block)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);

        ratatui::widgets::Widget::render(list, layout, self.buf);
    }

    pub fn render_all(&mut self) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(self.area);

        let info_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ])
            .split(layout[1]);

        self.render_memory(layout[0]);
        self.render_transaction_box(info_layout[0]);
        self.render_current_operation_box(info_layout[1]);
        self.render_operation_history(info_layout[2]);
    }
}

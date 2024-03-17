use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{
        block::{Position, Title},
        Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Table, TableState, Widget,
    },
};

use crate::state::{AppState, SlotStatus};

pub struct RenderData<'a> {
    pub area: Rect,
    pub buf: &'a mut Buffer,
    pub state: &'a mut AppState,
}

impl<'a> RenderData<'a> {
    fn render_memory(&mut self, layout: Rect) {
        // dbg!(&layout.width);
        let title = Title::from(" Trill ".bold());
        let instructions = Title::from(Line::from(vec![
            " Back ".into(),
            "<Left>".green().bold(),
            " Pause ".into(),
            "<Space>".yellow().bold(),
            " Forward ".into(),
            "<Right>".green().bold(),
            " Quit ".into(),
            "<Q> ".red().bold(),
        ]));
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::TOP)
            .border_set(border::THICK);

        let mut s = TableState::default();
        let mut rows: Vec<Row> = vec![];
        let mut row: Vec<Cell> = vec![];
        let width: usize = (layout.width / 2) as usize;
        let height: usize = (layout.height - 2) as usize;
        let mut first_slot: usize = self.state.table_beginning_index as usize * width;
        let mut range_ending = self.state.slots.len();

        if first_slot > self.state.slots.len() {
            self.state.table_beginning_index -= 1;
            first_slot = self.state.table_beginning_index as usize * width;
        }

        if width * height < self.state.slots.len() - first_slot {
            // need pagination
            range_ending = width * height;
        }

        for slot in first_slot..range_ending {
            match self.state.slots[slot] {
                SlotStatus::EMPTY => row.push(Cell::new("■").style(Style::new().gray())),
                SlotStatus::ACTIVE => row.push(Cell::new("■").style(Style::new().green())),
                SlotStatus::READING => row.push(Cell::new("■").style(Style::new().blue())),
                SlotStatus::WRITING => row.push(Cell::new("■").style(Style::new().red())),
                SlotStatus::INIT => (),
            }
            if slot % width == width - 1 || slot == self.state.slots.len() - 1 {
                rows.push(Row::new(row.clone()));
                row.clear();
            }
        }
        StatefulWidget::render(
            Table::new(rows, vec![Constraint::Length(1); width]).block(block),
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
        let success = match self.state.transaction_sucess {
            true => {
                Cell::new(self.state.transaction_sucess.to_string()).style(Style::new().green())
            }
            false => Cell::new(self.state.transaction_sucess.to_string()).style(Style::new().red()),
        };
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
                success,
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

        StatefulWidget::render(tx_info_table, layout, self.buf, &mut s);
    }

    fn render_current_operation_box(&mut self, layout: Rect) {
        let title = Title::from(" Operation info ".bold());
        let op_info_block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let mut vec = vec![];

        if let Some(op) = self.state.operation_to_render.as_ref() {
            let operation_code = match SlotStatus::from_opcode(op.operation) {
                SlotStatus::READING => Cell::new(op.operation.text()).style(Style::new().blue()),
                SlotStatus::WRITING => Cell::new(op.operation.text()).style(Style::new().red()),
                _ => Cell::new(op.operation.text()),
            };

            vec.extend(vec![
                Row::new(vec![
                    Cell::new("Code").style(Style::new().gray()),
                    operation_code,
                ]),
                Row::new(vec![
                    Cell::new("Gas cost").style(Style::new().gray()),
                    Cell::new(op.gas_cost.to_string()).style(Style::new().gray()),
                ]),
                Row::new(vec![
                    Cell::new("Gas remaining").style(Style::new().gray()),
                    Cell::new(op.remaining_gas.to_string()).style(Style::new().gray()),
                ]),
                Row::new(vec![
                    Cell::new("Program Counter").style(Style::new().gray()),
                    Cell::new(op.pc.to_string()).style(Style::new().gray()),
                ]),
            ]);
            let params = &op.params;
            for (key, value) in params.iter() {
                vec.push(Row::new(vec![
                    Cell::new(key.as_str()).style(Style::new().gray()),
                    Cell::new(value.as_str()).style(Style::new().green()),
                ]));
            }
        }

        let op_info_rows = vec;
        let op_info_table = Table::new(
            op_info_rows,
            [Constraint::Percentage(20), Constraint::Fill(1)],
        )
        .block(op_info_block);
        let mut s = TableState::default();

        StatefulWidget::render(op_info_table, layout, self.buf, &mut s);
    }

    fn render_operation_history(&mut self, layout: Rect) {
        let title = Title::from(" History ".bold());

        let instructions = Title::from(Line::from(vec![
            "<Up>".yellow().bold(),
            " Scroll ".into(),
            "<Down>".yellow().bold(),
        ]));

        let history_info_block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let items: Vec<Line> = self
            .state
            .operation_codes
            .iter()
            .map(|op| match SlotStatus::from_opcode(*op) {
                SlotStatus::READING => Line::from(op.text()).style(Style::new().blue()),
                SlotStatus::WRITING => Line::from(op.text()).style(Style::new().red()),
                _ => Line::default(),
            })
            .collect();

        let items_collection = Paragraph::new(items.clone())
            .scroll((self.state.history_vertical_scroll, 0))
            .block(history_info_block)
            .style(Style::default().fg(Color::White));

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut scrollbar_state =
            ScrollbarState::new(items.len()).position(self.state.history_vertical_scroll as usize);
        Widget::render(items_collection, layout, self.buf);
        StatefulWidget::render(
            scrollbar,
            layout.inner(&Margin {
                // using an inner vertical margin of 1 unit makes the scrollbar inside the block
                vertical: 1,
                horizontal: 0,
            }),
            self.buf,
            &mut scrollbar_state,
        );
    }

    pub fn render_all(&mut self) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(self.area);

        let info_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(40),
                Constraint::Percentage(50),
                Constraint::Percentage(10),
            ])
            .split(layout[1]);

        self.render_memory(layout[0]);
        self.render_transaction_box(info_layout[0]);
        self.render_current_operation_box(info_layout[1]);
        self.render_operation_history(info_layout[2]);
    }
}

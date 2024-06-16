use std::{thread::sleep, time::Duration};

use color_eyre::owo_colors::OwoColorize;
use itertools::Itertools;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    symbols::{self, border},
    text::Line,
    widgets::{
        block::{Position, Title},
        Axis, Block, Borders, Cell, Chart, Dataset, GraphType, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState, Widget,
    },
};

use crate::state::{AppState, SlotStatus};

pub struct RenderData<'a> {
    pub area: Rect,
    pub buf: &'a mut Buffer,
    pub state: &'a mut AppState,
}

impl<'a> RenderData<'a> {
    fn render_memory(&mut self, transaction_index: usize, layout: Rect) {
        let transaction_state = self.state.transaction_states[transaction_index].clone();
        let title = Title::from(" Trill ".bold());
        let instructions = Title::from(Line::from(vec![
            " Raw ".into(),
            "<F>".blue().bold(),
            " Up ".into(),
            "<W>".green().bold(),
            " Pause ".into(),
            "<Space>".yellow().bold(),
            " Down ".into(),
            "<S>".green().bold(),
            " Quit ".into(),
            "<Q> ".red().bold(),
        ]));
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(instructions.alignment(Alignment::Center).position(Position::Bottom))
            .borders(Borders::TOP)
            .border_set(border::THICK);

        let mut s = TableState::default();
        let mut constraints: Vec<Constraint> = vec![];
        let mut rows: Vec<Row> = vec![];
        let height: usize = (layout.height - 2) as usize;

        if self.state.display_memory_data {
            let mut first_slot: usize = self.state.table_beginning_index as usize;

            let operation_memory =
                &transaction_state.raw_data[(transaction_state.next_operation - 1) as usize].memory;

            if let Some(memory) = operation_memory {
                if first_slot >= memory.len() {
                    first_slot = memory.len() - 1;
                }
                let data = memory.into_iter().skip(first_slot).enumerate();
                for (index, slot) in data {
                    if index >= height {
                        break;
                    }
                    let mut row: Vec<Cell> =
                        vec![Cell::new((index + first_slot).to_string()).gray()];
                    for chunk in &slot.chars().chunks(2) {
                        let pair: String = chunk.collect();
                        match transaction_state.slots[index + first_slot] {
                            SlotStatus::EMPTY => row.push(Cell::new(pair).gray()),
                            SlotStatus::ACTIVE => row.push(Cell::new(pair).green()),
                            SlotStatus::READING => row.push(Cell::new(pair).blue()),
                            SlotStatus::WRITING => row.push(Cell::new(pair).red()),
                            SlotStatus::UNREAD => row.push(Cell::new(pair).magenta()),
                            SlotStatus::INIT => (),
                        }
                    }
                    rows.push(Row::new(row));
                }
            }

            constraints = vec![Constraint::Percentage(4)];
            constraints.extend(vec![Constraint::Percentage(3); 32]);
        } else {
            let mut row: Vec<Cell> = vec![];
            let width: usize = (layout.width / 2) as usize;
            let mut first_slot: usize = self.state.table_beginning_index as usize * width;
            let mut range_ending = transaction_state.slots.len();

            while first_slot > transaction_state.slots.len() {
                self.state.table_beginning_index -= 1;
                first_slot = self.state.table_beginning_index as usize * width;
            }

            if width * height < transaction_state.slots.len() - first_slot {
                range_ending = width * height;
            }

            for slot in first_slot..range_ending {
                match transaction_state.slots[slot] {
                    SlotStatus::EMPTY => row.push(Cell::new("■").gray()),
                    SlotStatus::ACTIVE => row.push(Cell::new("■").green()),
                    SlotStatus::READING => row.push(Cell::new("■").blue()),
                    SlotStatus::WRITING => row.push(Cell::new("■").red()),
                    SlotStatus::UNREAD => row.push(Cell::new("■").magenta()),
                    SlotStatus::INIT => (),
                }
                if slot % width == width - 1 || slot == transaction_state.slots.len() - 1 {
                    rows.push(Row::new(row.clone()));
                    row.clear();
                }
            }
            constraints = vec![Constraint::Length(1); width];
        }

        StatefulWidget::render(
            Table::new(rows, constraints).block(block),
            layout,
            self.buf,
            &mut s,
        );
    }

    fn render_transaction_box(&mut self, transaction_index: usize, layout: Rect) {
        let transaction_state = &self.state.transaction_states[transaction_index];
        let transaction = &transaction_state.transaction;
        let title = Title::from(" Transaction info ".bold());
        let tx_info_block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK);
        let success = match transaction_state.transaction_success {
            true => Cell::new(transaction_state.transaction_success.to_string())
                .style(Style::new().green()),
            false => Cell::new(transaction_state.transaction_success.to_string())
                .style(Style::new().red()),
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
            Row::new(vec![Cell::new("Success").style(Style::new().gray().bold()), success]),
            Row::new(vec![
                Cell::new("Gas used").style(Style::new().gray().bold()),
                Cell::new(transaction.gas.to_string()).style(Style::new().gray()),
            ]),
        ];
        let tx_info_table =
            Table::new(tx_info_rows, [Constraint::Percentage(20), Constraint::Fill(1)])
                .block(tx_info_block);
        let mut s = TableState::default();

        StatefulWidget::render(tx_info_table, layout, self.buf, &mut s);
    }

    fn render_current_operation_box(&mut self, transaction_index: usize, layout: Rect) {
        let transaction_state = &self.state.transaction_states[transaction_index];
        let title = Title::from(" Operation info ".bold());
        let op_info_block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let mut vec = vec![];
        let op = &transaction_state.operation_to_render;
        let operation_code = match SlotStatus::from_opcode(&op.operation) {
            SlotStatus::READING => Cell::new(op.operation.text()).blue(),
            SlotStatus::WRITING => Cell::new(op.operation.text()).red(),
            _ => Cell::new(op.operation.text()).yellow(),
        };

        vec.extend(vec![
            Row::new(vec![Cell::new("Code").style(Style::new().gray()), operation_code]),
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

        let op_info_rows = vec;
        let op_info_table =
            Table::new(op_info_rows, [Constraint::Percentage(20), Constraint::Fill(1)])
                .block(op_info_block);
        let mut s = TableState::default();

        StatefulWidget::render(op_info_table, layout, self.buf, &mut s);
    }

    fn render_operation_history(&mut self, transaction_index: usize, layout: Rect) {
        let transaction_state = &self.state.transaction_states[transaction_index];
        let title = Title::from(" History ".bold());

        let instructions = Title::from(Line::from(vec![
            "<Up>".yellow().bold(),
            " Scroll ".into(),
            "<Down>".yellow().bold(),
        ]));

        let history_info_block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(instructions.alignment(Alignment::Center).position(Position::Bottom))
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let items: Vec<Line> = transaction_state
            .operation_codes
            .iter()
            .map(|op| match SlotStatus::from_opcode(op) {
                SlotStatus::READING => Line::from(op.text()).style(Style::new().blue()),
                SlotStatus::WRITING => Line::from(op.text()).style(Style::new().red()),
                _ => Line::from(op.text()).style(Style::new().yellow()),
            })
            .collect();

        let items_collection = Paragraph::new(items.clone())
            .scroll((self.state.history_vertical_scroll, 0))
            .block(history_info_block)
            .style(Style::default().fg(Color::White));

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let height = layout.height - 2;
        if items.len() > (self.state.history_vertical_scroll + height - 1) as usize
            && items.len() >= height as usize
            && !self.state.pause
        {
            self.state.history_vertical_scroll += 1;
        }

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

    fn render_charts(&mut self, transaction_index: usize, layout: Rect) {
        let transaction_state = &self.state.transaction_states[transaction_index];
        let divided_space = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout);

        let write_title = Title::from(" Writes ".bold().red());

        let write_block = Block::default()
            .title(write_title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let read_title = Title::from(" Reads ".bold().blue());

        let read_block = Block::default()
            .title(read_title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let write_dataset = vec![Dataset::default()
            .name("Writes")
            .marker(symbols::Marker::Dot)
            .graph_type(GraphType::Line)
            .style(Style::default().red())
            .data(&transaction_state.write_dataset)];

        let read_dataset = vec![Dataset::default()
            .name("Reads")
            .marker(symbols::Marker::Dot)
            .graph_type(GraphType::Line)
            .style(Style::default().blue())
            .data(&transaction_state.read_dataset)];

        if transaction_state.write_dataset.len() > transaction_state.read_dataset.len() {
            dbg!("DISCREPANCY");
        }

        // Create the X axis and define its properties
        let x_axis = Axis::default()
            .style(Style::default().white())
            .bounds([0.0, transaction_state.write_dataset.len() as f64])
            .labels(vec!["0".into(), transaction_state.write_dataset.len().to_string().into()]);

        // Create the Y axis and define its properties
        let write_y_axis = Axis::default()
            .style(Style::default().white())
            .bounds([0.0, transaction_state.write_dataset.last().unwrap().1 as f64])
            .labels(vec![
                "0".into(),
                (transaction_state.write_dataset.last().unwrap().1).ceil().to_string().into(),
            ]);

        let read_y_axis = Axis::default()
            .style(Style::default().white())
            .bounds([0.0, transaction_state.read_dataset.last().unwrap().1 as f64])
            .labels(vec![
                "0".into(),
                (transaction_state.read_dataset.last().unwrap().1).ceil().to_string().into(),
            ]);

        // Create the chart and link all the parts together
        let write_chart = Chart::new(write_dataset)
            .block(Block::default().title("Writes"))
            .x_axis(x_axis.clone())
            .y_axis(write_y_axis.clone())
            .block(write_block);

        let read_chart = Chart::new(read_dataset)
            .block(Block::default().title("Reads"))
            .x_axis(x_axis)
            .y_axis(read_y_axis)
            .block(read_block);

        Widget::render(write_chart, divided_space[0], self.buf);
        Widget::render(read_chart, divided_space[1], self.buf);
    }

    /*
    ______________________________________________________________________
    |                                                                    |
    |                                                                    |
    |                                                                    |
    |                                                                    |
    |                           memory_box                               |
    |                      Height 50%, Width 100%                        |
    |                                                                    |
    |                                                                    |
    |                                                                    |
    |____________________________________________________________________|
    |                          |                          |              |
    |      transaction_box     |        opcode_box        |              |
    |      Height 25%          |        Height 25%        |              |
    |      Width 45%           |        Width 45%         |  history_box |
    |__________________________|__________________________|  Height 50%  |
    |                                                     |  Width 10%   |
    |                      charts_box                     |              |
    |                Height 25%, Width 90%                |              |
    |_____________________________________________________|______________|
    */
    fn render_normal(&mut self) {
        let half_divded_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(self.area);

        let (memory_box, bottom_layout) = (half_divded_area[0], half_divded_area[1]);

        let divided_bottom_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
            .split(bottom_layout);

        let (bottom_left_layout, history_box) =
            (divided_bottom_layout[0], divided_bottom_layout[1]);

        let divided_bottom_left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(bottom_left_layout);

        let (info_boxes_layout, charts_box) =
            (divided_bottom_left_layout[0], divided_bottom_left_layout[1]);

        let info_boxes = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(info_boxes_layout);

        let (transaction_box, opcode_box) = (info_boxes[0], info_boxes[1]);

        self.render_memory(0, memory_box);
        self.render_transaction_box(0, transaction_box);
        self.render_current_operation_box(0, opcode_box);
        self.render_operation_history(0, history_box);
        self.render_charts(0, charts_box);

        if self.state.help {
            // display help box
        }
    }

    /*
    ______________________________________________________________________
    |                                                                    |
    |                           memory0_box                              |
    |                      Height 25%, Width 100%                        |
    |____________________________________________________________________|
    |                                                                    |
    |                           memory1_box                              |
    |                      Height 25%, Width 100%                        |
    |____________________________________________________________________|
    |                                                     |              |
    |                                                     |              |
    |                                                     |              |
    |                     chart_box                       |  history_box |
    |                Height 25%, Width 90%                |  Height 50%  |
    |                                                     |  Width 10%   |
    |                                                     |              |
    |                                                     |              |
    |_____________________________________________________|______________|
    */
    fn render_versus(&mut self) {
        let half_divded_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(self.area);

        let (memory_layout, bottom_layout) = (half_divded_area[0], half_divded_area[1]);

        let divided_memory_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(memory_layout);

        let (memory0_box, memory1_box) = (divided_memory_layout[0], divided_memory_layout[1]);

        let divided_bottom_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
            .split(bottom_layout);

        let (bottom_left_layout, history_box) =
            (divided_bottom_layout[0], divided_bottom_layout[1]);

        let divided_bottom_left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(bottom_left_layout);

        let (info_boxes_layout, charts_box) =
            (divided_bottom_left_layout[0], divided_bottom_left_layout[1]);

        let info_boxes = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(info_boxes_layout);

        let (transaction_box, opcode_box) = (info_boxes[0], info_boxes[1]);

        self.render_memory(0, memory0_box);
        self.render_memory(1, memory1_box);
        self.render_chart(bottom_layout);

        if self.state.help {
            // display help box
        }
    }

    fn render_chart(&mut self, layout: Rect) {
        let transaction_zero = &self.state.transaction_states[0];
        let transaction_one = &self.state.transaction_states[1];

        let title = Title::from(" Reads & Writes ".bold().red());

        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let datasets = vec![
            // Tx0 writes
            Dataset::default()
                .name("Tx0 writes")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().red())
                .data(&transaction_zero.write_dataset),
            // Tx0 reads
            Dataset::default()
                .name("Tx0 Reads")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().blue())
                .data(&transaction_zero.read_dataset),
            // Tx1 writes
            Dataset::default()
                .name("Tx1 Writes")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().light_red())
                .data(&transaction_one.write_dataset),
            // Tx1 reads
            Dataset::default()
                .name("Tx1 Reads")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().light_blue())
                .data(&transaction_one.read_dataset),
        ];

        let mut x_axis_upper_bound : f64;
        if transaction_zero.write_dataset.len() as f64 >= transaction_one.write_dataset.len() as f64 {
            x_axis_upper_bound = transaction_zero.write_dataset.len() as f64;
        } else {
            x_axis_upper_bound = transaction_one.write_dataset.len() as f64;
        }

        let mut y_axis_upper_bound : f64;
        
        if transaction_zero.write_dataset.last().unwrap().1 as f64 >= transaction_one.write_dataset.last().unwrap().1 as f64 {
            y_axis_upper_bound = transaction_zero.write_dataset.last().unwrap().1 as f64;
        } else {
            y_axis_upper_bound = transaction_one.write_dataset.last().unwrap().1 as f64;
        }

        // Create the X axis and define its properties
        let x_axis = Axis::default()
            .style(Style::default().white())
            .bounds([0.0, x_axis_upper_bound])
            .labels(vec!["0".into(), x_axis_upper_bound.ceil().to_string().into()]);

        let y_axis = Axis::default()
            .style(Style::default().white())
            .bounds([0.0, y_axis_upper_bound])
            .labels(vec![
                "0".into(),
                y_axis_upper_bound.ceil().to_string().into(),
            ]);

        // Create the chart and link all the parts together
        let chart = Chart::new(datasets)
            .block(Block::default().title("Writes"))
            .x_axis(x_axis.clone())
            .y_axis(y_axis.clone())
            .block(block);

        Widget::render(chart, layout, self.buf);
    }

    pub fn render_all(&mut self) {
        match self.state.mode {
            crate::state::AppMode::VERSUS => self.render_versus(),
            crate::state::AppMode::NORMAL => self.render_normal(),
        }
    }
}

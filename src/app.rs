use std::io;

use crate::render::RenderData;
use crate::state::AppState;
use crate::tui::{self, Event};

use alloy::primitives::TxHash;
use crossterm::event::KeyCode::Char;
use crossterm::event::KeyEvent;
use ratatui::prelude::*;
use tracing::{event, Level};

#[derive(Debug)]
pub struct App {
    iteration: u64,
    forward: bool, // false for going back one iteration, true for going forward in the processing
    pause: bool,   // false for not pause and true for pause
    exit: bool,
    scroll_table: Option<bool>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            iteration: 1,
            exit: false,
            forward: true, // go forward
            pause: false,
            scroll_table: None,
        }
    }
}
impl App {
    /// runs the application's main loop until the user quits
    pub async fn run(&mut self, transaction: TxHash) -> color_eyre::Result<()> {
        let mut tui = tui::Tui::new()?
            .tick_rate(1.0) // 4 ticks per second
            .frame_rate(4.0); // 30 frames per second

        tui.enter()?; // Starts event handler, enters raw mode, enters alternate screen
        let mut app_state = AppState::default();
        loop {
            if let Some(scroll_direction) = self.scroll_table {
                if !scroll_direction && app_state.table_beginning_index > 0 {
                    // Go up
                    app_state.table_beginning_index -= 1;
                } else if scroll_direction {
                    // Go down
                    app_state.table_beginning_index += 1;
                }

                self.scroll_table = None;
            }

            app_state = AppState::run(
                app_state,
                transaction,
                self.iteration,
                self.forward,
                self.pause,
            )
            .await?;
            event!(
                Level::INFO,
                "FROM RUN 1 APP.RS: {}",
                app_state.write_dataset.len()
            );

            if let Some(evt) = tui.next().await {
                match evt {
                    Event::Render => {
                        tui.draw(|f| {
                            event!(
                                Level::INFO,
                                "FROM RUN APP.RS: {}",
                                app_state.write_dataset.len()
                            );
                            self.render_frame(f, &mut app_state);
                        })?;
                    }
                    Event::Key(key) => {
                        self.handle_event(key, &mut app_state);
                    }
                    _ => {
                        event!(
                            Level::INFO,
                            "PASS"
                        );
                        event!(
                            Level::INFO,
                            "{:?}", &evt
                        );
                    }
                }
            };
            if self.exit {
                break;
            }
        }

        tui.exit()?; // stops event handler, exits raw mode, exits alternate screen

        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame, mut state: &mut AppState) {
        event!(
            Level::INFO,
            "FROM RENDER FRAME APP.RS: {}",
            state.write_dataset.len()
        );
        frame.render_stateful_widget(self, frame.size(), &mut state);
    }

    fn handle_event(&mut self, key: KeyEvent, state: &mut AppState) {
        match key.code {
            Char('q') => self.exit = true,
            crossterm::event::KeyCode::Left => self.forward = false,
            crossterm::event::KeyCode::Right => self.forward = true,
            crossterm::event::KeyCode::Down => {
                state.history_vertical_scroll = state.history_vertical_scroll + 1
            }
            crossterm::event::KeyCode::Up => {
                if state.history_vertical_scroll > 0 {
                    state.history_vertical_scroll -= 1;
                }
            }
            Char(' ') => self.pause = !self.pause,
            Char('w') => self.scroll_table = Some(false),
            Char('s') => self.scroll_table = Some(true),
            _ => {}
        }
    }
}

impl StatefulWidget for &App {
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        let mut render_data = RenderData { area, buf, state };
        render_data.render_all();
    }
    type State = AppState;
}

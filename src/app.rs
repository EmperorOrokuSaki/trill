use crate::render::RenderData;
use crate::state::AppState;
use crate::tui::{self, Event};

use alloy::primitives::TxHash;
use crossterm::event::KeyCode::Char;
use crossterm::event::KeyEvent;
use ratatui::prelude::*;

#[derive(Debug)]
pub struct App {
    /// Number of operations to process with each frame
    iteration: u64,
    /// false for going back one iteration, true for going forward in the processing
    forward: bool,
    /// false for not pause and true for pause
    pause: bool,
    /// main loop's exit code
    exit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            iteration: 1,
            exit: false,
            forward: true, // go forward
            pause: false,
        }
    }
}

impl App {
    /// runs the application's main loop until the user quits
    pub async fn run(
        &mut self,
        transaction: TxHash,
        fps: f64,
        iteration: u64,
        rpc: String,
    ) -> color_eyre::Result<()> {
        let mut tui = tui::Tui::new()?.frame_rate(fps);
        self.iteration = iteration;

        tui.enter()?; // Starts event handler, enters raw mode, enters alternate screen
        let mut app_state = AppState::default();

        // Main loop begins here
        loop {
            // Handle main table (memory slots) scrolling
            app_state = AppState::run(
                app_state,
                transaction,
                self.iteration,
                self.forward,
                self.pause,
                &rpc,
            )
            .await?;

            if let Some(evt) = tui.next().await {
                match evt {
                    Event::Render => {
                        tui.draw(|f| {
                            self.render_frame(f, &mut app_state);
                        })?;
                    }
                    Event::Key(key) => {
                        self.handle_event(key, &mut app_state);
                    }
                    _ => {}
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
            Char('w') => {
                if state.table_beginning_index > 0 {
                    // Go up
                    state.table_beginning_index -= 1;
                }
            },
            Char('s') => state.table_beginning_index += 1,
            Char('h') => state.help = !state.help,
            _ => {}
        }
    }

    // fn handle_table_scrolling
}

impl StatefulWidget for &App {
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        let mut render_data = RenderData { area, buf, state };
        render_data.render_all();
    }
    type State = AppState;
}

use std::io;

use crate::render::RenderData;
use crate::state::AppState;
use crate::tui::{self, Event};

use crossterm::event::KeyCode::Char;
use ratatui::prelude::*;

#[derive(Debug)]
pub struct App {
    iteration: u64,
    forward: bool, // false for going back one iteration, true for going forward in the processing
    pause: bool,   // false for not pause and true for pause
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
    pub async fn run(&mut self) -> color_eyre::Result<()> {
        let mut tui = tui::Tui::new()?
            .tick_rate(2.0) // 4 ticks per second
            .frame_rate(1.0); // 30 frames per second

        tui.enter()?; // Starts event handler, enters raw mode, enters alternate screen
        let mut app_state = AppState::default();
        loop {
            app_state = AppState::run(app_state, self.iteration, self.forward, self.pause).await?;

            tui.draw(|f| {
                // Deref allows calling tui.terminal.draw
                self.render_frame(f, &mut app_state);
            })?;
            self.forward = true;
            if let Some(evt) = tui.next().await {
                // tui.next().await blocks till next event
                self.handle_event(evt, &mut app_state)?;
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

    fn handle_event(&mut self, event: Event, state: &mut AppState) -> io::Result<()> {
        if let Event::Key(key) = event {
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
                _ => {}
            }
        }
        Ok(())
    }
}

impl StatefulWidget for &App {
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        let mut render_data = RenderData { area, buf, state };
        render_data.render_all();
    }
    type State = AppState;
}

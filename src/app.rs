use crate::{
    render::RenderData,
    state::AppState,
    tui::{self, Event},
};

use color_eyre::eyre;
use crossterm::event::{KeyCode::Char, KeyEvent};
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
        app_state: &mut AppState,
        fps: f64,
        iteration: u64,
    ) -> color_eyre::Result<(), eyre::Error> {
        let mut tui = tui::Tui::new()?.frame_rate(fps);
        self.iteration = iteration;

        tui.enter()?; // Starts event handler, enters raw mode, enters alternate screen

        // Main loop begins here
        loop {
            app_state.run(self.iteration, self.forward, self.pause).await?;

            if let Some(evt) = tui.next().await {
                match evt {
                    Event::Render => {
                        tui.draw(|f| {
                            self.render_frame(f, app_state);
                        })?;
                    }
                    Event::Key(key) => {
                        self.handle_event(key, app_state);
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

    fn render_frame(&self, frame: &mut Frame, state: &mut AppState) {
        frame.render_stateful_widget(self, frame.size(), state);
    }

    fn handle_event(&mut self, key: KeyEvent, state: &mut AppState) {
        match key.code {
            Char(c) => {
                match c.to_ascii_lowercase() {
                    'q' => self.exit = true,
                    ' ' => self.pause = !self.pause,
                    'w' => {
                        if state.table_beginning_index > 0 {
                            // Go up
                            state.table_beginning_index -= 1;
                        }
                    }
                    's' => state.table_beginning_index += 1,
                    'h' => state.help = !state.help,
                    'f' => state.display_memory_data = !state.display_memory_data,
                    _ => {}
                }
            }
            crossterm::event::KeyCode::Left => self.forward = false,
            crossterm::event::KeyCode::Right => self.forward = true,
            crossterm::event::KeyCode::Down => state.history_vertical_scroll += 1,
            crossterm::event::KeyCode::Up => {
                if state.history_vertical_scroll > 0 {
                    state.history_vertical_scroll -= 1;
                }
            }
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

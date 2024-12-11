use log::info;

use crate::errors::ErrorButtons;
use crate::errors::ErrorFerris;

use super::App;

impl App {
    pub(super) fn quit(&mut self, ctx: &egui::Context) {
        // If closing.
        // Used to be `eframe::App::on_close_event(&mut self) -> bool`.
        let close_signal = ctx.input(|input| {
            use egui::viewport::ViewportCommand;

            if !input.viewport().close_requested() {
                return None;
            }
            info!("quit");
            if self.state.gupax.auto.ask_before_quit {
                // If we're already on the [ask_before_quit] screen and
                // the user tried to exit again, exit.
                if self.error_state.quit_twice {
                    if self.state.gupax.auto.save_before_quit {
                        self.save_before_quit();
                    }
                    return Some(ViewportCommand::Close);
                }
                // Else, set the error
                self.error_state
                    .set("", ErrorFerris::Oops, ErrorButtons::StayQuit);
                self.error_state.quit_twice = true;
                Some(ViewportCommand::CancelClose)
            // Else, just quit.
            } else {
                if self.state.gupax.auto.save_before_quit {
                    self.save_before_quit();
                }
                Some(ViewportCommand::Close)
            }
        });
        // This will either:
        // 1. Cancel a close signal
        // 2. Close the program
        if let Some(cmd) = close_signal {
            ctx.send_viewport_cmd(cmd);
        }
    }
}

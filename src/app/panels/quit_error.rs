use std::process::exit;

use crate::app::keys::KeyPressed;
use crate::disk::node::Node;
use crate::disk::state::State;
use crate::utils::constants::*;
use crate::utils::errors::ErrorState;
use crate::utils::ferris::*;
use crate::utils::macros::{arc_mut, flip, lock, lock2};
use crate::utils::resets::{reset_nodes, reset_state};
use crate::utils::sudo::SudoState;
use egui::TextStyle::Name;
use egui::*;

impl crate::app::App {
    pub(in crate::app) fn quit_error_panel(
        &mut self,
        ctx: &egui::Context,
        p2pool_is_alive: bool,
        xmrig_is_alive: bool,
        key: &KeyPressed,
    ) {
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // Set width/height/font
                let width = self.size.x;
                let height = self.size.y / 4.0;
                ui.style_mut().override_text_style = Some(Name("MonospaceLarge".into()));

                // Display ferris
                use crate::utils::errors::ErrorButtons;
                use crate::utils::errors::ErrorButtons::*;
                use crate::utils::errors::ErrorFerris;
                use crate::utils::errors::ErrorFerris::*;
                let ferris = match self.error_state.ferris {
                    Happy => Image::from_bytes("bytes://happy.png", FERRIS_HAPPY),
                    Cute => Image::from_bytes("bytes://cute.png", FERRIS_CUTE),
                    Oops => Image::from_bytes("bytes://oops.png", FERRIS_OOPS),
                    Error => Image::from_bytes("bytes://error.png", FERRIS_ERROR),
                    Panic => Image::from_bytes("bytes://panic.png", FERRIS_PANIC),
                    ErrorFerris::Sudo => Image::from_bytes("bytes://panic.png", FERRIS_SUDO),
                };

                match self.error_state.buttons {
                    ErrorButtons::Debug => ui.add_sized(
                        [width, height / 4.0],
                        Label::new("--- Debug Info ---\n\nPress [ESC] to quit"),
                    ),
                    _ => ui.add_sized(Vec2::new(width, height), ferris),
                };

                // Error/Quit screen
                match self.error_state.buttons {
                    StayQuit => {
                        let mut text = "".to_string();
                        if *lock2!(self.update, updating) {
                            text = format!(
                                "{}\nUpdate is in progress...! Quitting may cause file corruption!",
                                text
                            );
                        }
                        if p2pool_is_alive {
                            text = format!("{}\nP2Pool is online...!", text);
                        }
                        if xmrig_is_alive {
                            text = format!("{}\nXMRig is online...!", text);
                        }
                        ui.add_sized(
                            [width, height],
                            Label::new("--- Are you sure you want to quit? ---"),
                        );
                        ui.add_sized([width, height], Label::new(text))
                    }
                    ResetState => {
                        ui.add_sized(
                            [width, height],
                            Label::new(format!(
                                "--- Gupax has encountered an error! ---\n{}",
                                &self.error_state.msg
                            )),
                        );
                        ui.add_sized(
                            [width, height],
                            Label::new("Reset Gupax state? (Your settings)"),
                        )
                    }
                    ResetNode => {
                        ui.add_sized(
                            [width, height],
                            Label::new(format!(
                                "--- Gupax has encountered an error! ---\n{}",
                                &self.error_state.msg
                            )),
                        );
                        ui.add_sized([width, height], Label::new("Reset the manual node list?"))
                    }
                    ErrorButtons::Sudo => {
                        let text = format!(
                            "Why does XMRig need admin privilege?\n{}",
                            XMRIG_ADMIN_REASON
                        );
                        let height = height / 4.0;
                        ui.add_sized(
                            [width, height],
                            Label::new(format!(
                                "--- Gupax needs sudo/admin privilege for XMRig! ---\n{}",
                                &self.error_state.msg
                            )),
                        );
                        ui.style_mut().override_text_style = Some(Name("MonospaceSmall".into()));
                        ui.add_sized([width / 2.0, height], Label::new(text));
                        ui.add_sized(
                            [width, height],
                            Hyperlink::from_label_and_url(
                                "Click here for more info.",
                                "https://xmrig.com/docs/miner/randomx-optimization-guide",
                            ),
                        )
                    }
                    Debug => {
                        egui::Frame::none().fill(DARK_GRAY).show(ui, |ui| {
                            let width = ui.available_width();
                            let height = ui.available_height();
                            egui::ScrollArea::vertical()
                                .max_width(width)
                                .max_height(height)
                                .auto_shrink([false; 2])
                                .show_viewport(ui, |ui, _| {
                                    ui.add_sized(
                                        [width - 20.0, height],
                                        TextEdit::multiline(&mut self.error_state.msg.as_str()),
                                    );
                                });
                        });
                        ui.label("")
                    }
                    _ => {
                        match self.error_state.ferris {
                            Panic => ui.add_sized(
                                [width, height],
                                Label::new("--- Gupax has encountered an unrecoverable error! ---"),
                            ),
                            Happy => ui.add_sized([width, height], Label::new("--- Success! ---")),
                            _ => ui.add_sized(
                                [width, height],
                                Label::new("--- Gupax has encountered an error! ---"),
                            ),
                        };
                        let height = height / 2.0;
                        // Show GitHub rant link for Windows admin problems.
                        if cfg!(windows) && self.error_state.buttons == ErrorButtons::WindowsAdmin {
                            ui.add_sized([width, height], Hyperlink::from_label_and_url(
								"[Why does Gupax need to be Admin? (on Windows)]",
								"https://github.com/hinto-janai/gupax/tree/main/src#why-does-gupax-need-to-be-admin-on-windows"
							));
                            ui.add_sized([width, height], Label::new(&self.error_state.msg))
                        } else {
                            ui.add_sized([width, height], Label::new(&self.error_state.msg))
                        }
                    }
                };
                let height = ui.available_height();

                match self.error_state.buttons {
                    YesNo => {
                        if ui
                            .add_sized([width, height / 2.0], Button::new("Yes"))
                            .clicked()
                        {
                            self.error_state.reset()
                        }
                        // If [Esc] was pressed, assume [No]
                        if key.is_esc()
                            || ui
                                .add_sized([width, height / 2.0], Button::new("No"))
                                .clicked()
                        {
                            exit(0);
                        }
                    }
                    StayQuit => {
                        // If [Esc] was pressed, assume [Stay]
                        if key.is_esc()
                            || ui
                                .add_sized([width, height / 2.0], Button::new("Stay"))
                                .clicked()
                        {
                            self.error_state = ErrorState::new();
                        }
                        if ui
                            .add_sized([width, height / 2.0], Button::new("Quit"))
                            .clicked()
                        {
                            if self.state.gupax.save_before_quit {
                                self.save_before_quit();
                            }
                            exit(0);
                        }
                    }
                    // This code handles the [state.toml/node.toml] resetting, [panic!]'ing if it errors once more
                    // Another error after this either means an IO error or permission error, which Gupax can't fix.
                    // [Yes/No] buttons
                    ResetState => {
                        if ui
                            .add_sized([width, height / 2.0], Button::new("Yes"))
                            .clicked()
                        {
                            match reset_state(&self.state_path) {
                                Ok(_) => match State::get(&self.state_path) {
                                    Ok(s) => {
                                        self.state = s;
                                        self.og = arc_mut!(self.state.clone());
                                        self.error_state.set(
                                            "State read OK",
                                            ErrorFerris::Happy,
                                            ErrorButtons::Okay,
                                        );
                                    }
                                    Err(e) => self.error_state.set(
                                        format!("State read fail: {}", e),
                                        ErrorFerris::Panic,
                                        ErrorButtons::Quit,
                                    ),
                                },
                                Err(e) => self.error_state.set(
                                    format!("State reset fail: {}", e),
                                    ErrorFerris::Panic,
                                    ErrorButtons::Quit,
                                ),
                            };
                        }
                        if key.is_esc()
                            || ui
                                .add_sized([width, height / 2.0], Button::new("No"))
                                .clicked()
                        {
                            self.error_state.reset()
                        }
                    }
                    ResetNode => {
                        if ui
                            .add_sized([width, height / 2.0], Button::new("Yes"))
                            .clicked()
                        {
                            match reset_nodes(&self.node_path) {
                                Ok(_) => match Node::get(&self.node_path) {
                                    Ok(s) => {
                                        self.node_vec = s;
                                        self.og_node_vec.clone_from(&self.node_vec);
                                        self.error_state.set(
                                            "Node read OK",
                                            ErrorFerris::Happy,
                                            ErrorButtons::Okay,
                                        );
                                    }
                                    Err(e) => self.error_state.set(
                                        format!("Node read fail: {}", e),
                                        ErrorFerris::Panic,
                                        ErrorButtons::Quit,
                                    ),
                                },
                                Err(e) => self.error_state.set(
                                    format!("Node reset fail: {}", e),
                                    ErrorFerris::Panic,
                                    ErrorButtons::Quit,
                                ),
                            };
                        }
                        if key.is_esc()
                            || ui
                                .add_sized([width, height / 2.0], Button::new("No"))
                                .clicked()
                        {
                            self.error_state.reset()
                        }
                    }
                    ErrorButtons::Sudo => {
                        let sudo_width = width / 10.0;
                        let height = ui.available_height() / 4.0;
                        let mut sudo = lock!(self.sudo);
                        let hide = sudo.hide;
                        if sudo.testing {
                            ui.add_sized([width, height], Spinner::new().size(height));
                            ui.set_enabled(false);
                        } else {
                            ui.add_sized([width, height], Label::new(&sudo.msg));
                        }
                        ui.add_space(height);
                        let height = ui.available_height() / 5.0;
                        // Password input box with a hider.
                        ui.horizontal(|ui| {
                            let response = ui.add_sized(
                                [sudo_width * 8.0, height],
                                TextEdit::hint_text(
                                    TextEdit::singleline(&mut sudo.pass).password(hide),
                                    PASSWORD_TEXT,
                                ),
                            );
                            let box_width = (ui.available_width() / 2.0) - 5.0;
                            if (response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)))
                                || ui
                                    .add_sized([box_width, height], Button::new("Enter"))
                                    .on_hover_text(PASSWORD_ENTER)
                                    .clicked()
                            {
                                response.request_focus();
                                if !sudo.testing {
                                    SudoState::test_sudo(
                                        self.sudo.clone(),
                                        &self.helper.clone(),
                                        &self.state.xmrig,
                                        &self.state.gupax.absolute_xmrig_path,
                                    );
                                }
                            }
                            let color = if hide { BLACK } else { BRIGHT_YELLOW };
                            if ui
                                .add_sized(
                                    [box_width, height],
                                    Button::new(RichText::new("👁").color(color)),
                                )
                                .on_hover_text(PASSWORD_HIDE)
                                .clicked()
                            {
                                flip!(sudo.hide);
                            }
                        });
                        if (key.is_esc() && !sudo.testing)
                            || ui
                                .add_sized([width, height * 4.0], Button::new("Leave"))
                                .on_hover_text(PASSWORD_LEAVE)
                                .clicked()
                        {
                            self.error_state.reset();
                        };
                        // If [test_sudo()] finished, reset error state.
                        if sudo.success {
                            self.error_state.reset();
                        }
                    }
                    crate::app::ErrorButtons::Okay | crate::app::ErrorButtons::WindowsAdmin => {
                        if key.is_esc()
                            || ui.add_sized([width, height], Button::new("Okay")).clicked()
                        {
                            self.error_state.reset();
                        }
                    }
                    Debug => {
                        if key.is_esc() {
                            self.error_state.reset();
                        }
                    }
                    Quit => {
                        if ui.add_sized([width, height], Button::new("Quit")).clicked() {
                            exit(1);
                        }
                    }
                }
            })
        });
    }
}

use egui::{Key, Modifiers};
use log::info;

use crate::{disk::status::Submenu, utils::macros::flip};

use super::{App, Tab};

//---------------------------------------------------------------------------------------------------- [Pressed] enum
// These represent the keys pressed during the frame.
// I could use egui's [Key] but there is no option for
// a [None] and wrapping [key_pressed] like [Option<egui::Key>]
// meant that I had to destructure like this:
//     if let Some(egui::Key)) = key_pressed { /* do thing */ }
//
// That's ugly, so these are used instead so a simple compare can be used.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum KeyPressed {
    F11,
    Up,
    Down,
    Esc,
    Z,
    X,
    C,
    V,
    S,
    R,
    D,
    None,
}

impl KeyPressed {
    #[inline]
    pub(super) fn is_f11(&self) -> bool {
        *self == Self::F11
    }
    #[inline]
    pub(super) fn is_z(&self) -> bool {
        *self == Self::Z
    }
    #[inline]
    pub(super) fn is_x(&self) -> bool {
        *self == Self::X
    }
    #[inline]
    pub(super) fn is_up(&self) -> bool {
        *self == Self::Up
    }
    #[inline]
    pub(super) fn is_down(&self) -> bool {
        *self == Self::Down
    }
    #[inline]
    pub(super) fn is_esc(&self) -> bool {
        *self == Self::Esc
    }
    #[inline]
    pub(super) fn is_s(&self) -> bool {
        *self == Self::S
    }
    #[inline]
    pub(super) fn is_r(&self) -> bool {
        *self == Self::R
    }
    #[inline]
    pub(super) fn is_d(&self) -> bool {
        *self == Self::D
    }
    #[inline]
    pub(super) fn is_c(&self) -> bool {
        *self == Self::C
    }
    #[inline]
    pub(super) fn is_v(&self) -> bool {
        *self == Self::V
    }
    // #[inline]
    // pub(super) fn is_none(&self) -> bool {
    //     *self == Self::None
    // }
}

impl App {
    pub fn keys_handle(&mut self, ctx: &egui::Context) -> (KeyPressed, bool) {
        // If [F11] was pressed, reverse [fullscreen] bool
        let key: KeyPressed = ctx.input_mut(|input| {
            if input.consume_key(Modifiers::NONE, Key::F11) {
                KeyPressed::F11
            } else if input.consume_key(Modifiers::NONE, Key::Z) {
                KeyPressed::Z
            } else if input.consume_key(Modifiers::NONE, Key::X) {
                KeyPressed::X
            } else if input.consume_key(Modifiers::NONE, Key::C) {
                KeyPressed::C
            } else if input.consume_key(Modifiers::NONE, Key::V) {
                KeyPressed::V
            } else if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
                KeyPressed::Up
            } else if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
                KeyPressed::Down
            } else if input.consume_key(Modifiers::NONE, Key::Escape) {
                KeyPressed::Esc
            } else if input.consume_key(Modifiers::NONE, Key::S) {
                KeyPressed::S
            } else if input.consume_key(Modifiers::NONE, Key::R) {
                KeyPressed::R
            } else if input.consume_key(Modifiers::NONE, Key::D) {
                KeyPressed::D
            } else {
                KeyPressed::None
            }
        });
        // Check if egui wants keyboard input.
        // This prevents keyboard shortcuts from clobbering TextEdits.
        // (Typing S in text would always [Save] instead)
        let wants_input = ctx.wants_keyboard_input();

        if key.is_f11() {
            if ctx.input(|i| i.viewport().maximized == Some(true)) {
                info!("fullscreen bool");
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
            }
        // Change Tabs LEFT
        } else if key.is_z() && !wants_input {
            match self.tab {
                Tab::About => self.tab = Tab::Xvb,
                Tab::Status => self.tab = Tab::About,
                Tab::Gupax => self.tab = Tab::Status,
                Tab::P2pool => self.tab = Tab::Gupax,
                Tab::Xmrig => self.tab = Tab::P2pool,
                Tab::Xvb => self.tab = Tab::Xmrig,
            };
        // Change Tabs RIGHT
        } else if key.is_x() && !wants_input {
            match self.tab {
                Tab::About => self.tab = Tab::Status,
                Tab::Status => self.tab = Tab::Gupax,
                Tab::Gupax => self.tab = Tab::P2pool,
                Tab::P2pool => self.tab = Tab::Xmrig,
                Tab::Xmrig => self.tab = Tab::Xvb,
                Tab::Xvb => self.tab = Tab::About,
            };
        // Change Submenu LEFT
        } else if key.is_c() && !wants_input {
            match self.tab {
                Tab::Status => match self.state.status.submenu {
                    Submenu::Processes => self.state.status.submenu = Submenu::Benchmarks,
                    Submenu::P2pool => self.state.status.submenu = Submenu::Processes,
                    Submenu::Benchmarks => self.state.status.submenu = Submenu::P2pool,
                },
                Tab::Gupax => flip!(self.state.gupax.simple),
                Tab::P2pool => flip!(self.state.p2pool.simple),
                Tab::Xmrig => flip!(self.state.xmrig.simple),
                _ => (),
            };
        // Change Submenu RIGHT
        } else if key.is_v() && !wants_input {
            match self.tab {
                Tab::Status => match self.state.status.submenu {
                    Submenu::Processes => self.state.status.submenu = Submenu::P2pool,
                    Submenu::P2pool => self.state.status.submenu = Submenu::Benchmarks,
                    Submenu::Benchmarks => self.state.status.submenu = Submenu::Processes,
                },
                Tab::Gupax => flip!(self.state.gupax.simple),
                Tab::P2pool => flip!(self.state.p2pool.simple),
                Tab::Xmrig => flip!(self.state.xmrig.simple),
                _ => (),
            };
        }
        (key, wants_input)
    }
}

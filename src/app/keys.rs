// Gupax
//
// Copyright (c) 2024-2025 Cyrix126
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use egui::{Key, Modifiers};
use log::info;

use crate::{
    app::submenu_enum::{SubmenuP2pool, SubmenuStatus},
    utils::macros::flip,
};

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
            let tabs = Tab::from_show_processes(&self.state.gupax.show_processes);
            let index = tabs
                .iter()
                .position(|t| *t == self.tab)
                .expect("can't be on a hidden tab");
            self.tab = if (index as i32 - 1) < 0 {
                tabs.last()
                    .expect("there is always 3 tabs that can not be hidden")
                    .to_owned()
            } else {
                tabs[index - 1]
            };
        // Change Tabs RIGHT
        } else if key.is_x() && !wants_input {
            let tabs = Tab::from_show_processes(&self.state.gupax.show_processes);
            let index = tabs
                .iter()
                .position(|t| *t == self.tab)
                .expect("can't be on a hidden tab");
            self.tab = if (index + 1) == tabs.len() {
                tabs[0]
            } else {
                tabs[index + 1]
            };
        // Change Submenu LEFT
        } else if key.is_c() && !wants_input {
            match self.tab {
                Tab::Status => match self.state.status.submenu {
                    SubmenuStatus::Processes => {
                        self.state.status.submenu = SubmenuStatus::Benchmarks
                    }
                    SubmenuStatus::P2pool => self.state.status.submenu = SubmenuStatus::Processes,
                    SubmenuStatus::Benchmarks => self.state.status.submenu = SubmenuStatus::P2pool,
                },
                Tab::Gupax => flip!(self.state.gupax.simple),
                Tab::Node => flip!(self.state.node.simple),
                Tab::P2pool => match self.state.p2pool.submenu {
                    SubmenuP2pool::Simple => self.state.p2pool.submenu = SubmenuP2pool::Crawler,
                    SubmenuP2pool::Advanced => self.state.p2pool.submenu = SubmenuP2pool::Simple,
                    SubmenuP2pool::Crawler => self.state.p2pool.submenu = SubmenuP2pool::Advanced,
                },
                Tab::Xmrig => flip!(self.state.xmrig.simple),
                Tab::XmrigProxy => flip!(self.state.xmrig_proxy.simple),
                Tab::Xvb => flip!(self.state.xvb.simple),
                Tab::About => (),
            };
        // Change Submenu RIGHT
        } else if key.is_v() && !wants_input {
            match self.tab {
                Tab::Status => match self.state.status.submenu {
                    SubmenuStatus::Processes => self.state.status.submenu = SubmenuStatus::P2pool,
                    SubmenuStatus::P2pool => self.state.status.submenu = SubmenuStatus::Benchmarks,
                    SubmenuStatus::Benchmarks => {
                        self.state.status.submenu = SubmenuStatus::Processes
                    }
                },
                Tab::Gupax => flip!(self.state.gupax.simple),
                Tab::P2pool => match self.state.p2pool.submenu {
                    SubmenuP2pool::Simple => self.state.p2pool.submenu = SubmenuP2pool::Advanced,
                    SubmenuP2pool::Advanced => self.state.p2pool.submenu = SubmenuP2pool::Crawler,
                    SubmenuP2pool::Crawler => self.state.p2pool.submenu = SubmenuP2pool::Simple,
                },
                Tab::Xmrig => flip!(self.state.xmrig.simple),
                Tab::XmrigProxy => flip!(self.state.xmrig_proxy.simple),
                Tab::Xvb => flip!(self.state.xvb.simple),
                Tab::Node => flip!(self.state.node.simple),
                Tab::About => (),
            };
        }
        (key, wants_input)
    }
}

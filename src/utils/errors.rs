use std::sync::{Arc, Mutex};

use sysinfo::System;

use crate::{
    components::update::{NODE_BINARY, P2POOL_BINARY, XMRIG_BINARY, XMRIG_PROXY_BINARY},
    helper::ProcessName,
};

use super::sudo::SudoState;

//---------------------------------------------------------------------------------------------------- [ErrorState] struct
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ErrorButtons {
    YesNo,
    StayQuit,
    ResetState,
    ResetNode,
    Okay,
    Quit,
    Sudo,
    WindowsAdmin,
    Debug,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorFerris {
    Happy,
    Cute,
    Oops,
    Error,
    Panic,
    Sudo,
}

pub struct ErrorState {
    pub error: bool,           // Is there an error?
    pub msg: String,           // What message to display?
    pub ferris: ErrorFerris,   // Which ferris to display?
    pub buttons: ErrorButtons, // Which buttons to display?
    pub quit_twice: bool, // This indicates the user tried to quit on the [ask_before_quit] screen
}

impl Default for ErrorState {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorState {
    pub fn new() -> Self {
        Self {
            error: false,
            msg: "Unknown Error".to_string(),
            ferris: ErrorFerris::Oops,
            buttons: ErrorButtons::Okay,
            quit_twice: false,
        }
    }

    // Convenience function to enable the [App] error state
    pub fn set(&mut self, msg: impl Into<String>, ferris: ErrorFerris, buttons: ErrorButtons) {
        if self.error {
            // If a panic error is already set and there isn't an [Okay] confirm or another [Panic], return
            if self.ferris == ErrorFerris::Panic
                && (buttons != ErrorButtons::Okay || ferris != ErrorFerris::Panic)
            {
                return;
            }
        }
        *self = Self {
            error: true,
            msg: msg.into(),
            ferris,
            buttons,
            quit_twice: false,
        };
    }

    // Just sets the current state to new, resetting it.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    // Instead of creating a whole new screen and system, this (ab)uses ErrorState
    // to ask for the [sudo] when starting XMRig. Yes, yes I know, it's called "ErrorState"
    // but rewriting the UI code and button stuff might be worse.
    // It also resets the current [SudoState]
    pub fn ask_sudo(&mut self, state: &Arc<Mutex<SudoState>>) {
        *self = Self {
            error: true,
            msg: String::new(),
            ferris: ErrorFerris::Sudo,
            buttons: ErrorButtons::Sudo,
            quit_twice: false,
        };
        SudoState::reset(state)
    }
}

pub fn process_running(process_name: ProcessName) -> bool {
    let name = match process_name {
        ProcessName::Node => NODE_BINARY,
        ProcessName::P2pool => P2POOL_BINARY,
        ProcessName::Xmrig => XMRIG_BINARY,
        ProcessName::XmrigProxy => XMRIG_PROXY_BINARY,
        ProcessName::Xvb => {
            // XvB does not exist as a process outside of Gupaxx (not yet anyway);
            return false;
        }
    };
    let s = System::new_all();
    if s.processes_by_exact_name(name.as_ref()).next().is_some() {
        return true;
    }
    false
}

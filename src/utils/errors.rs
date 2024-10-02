use std::sync::{Arc, Mutex};

#[cfg(target_os = "windows")]
use sysinfo::System;

#[cfg(target_os = "windows")]
use crate::helper::ProcessName;

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

#[cfg(target_os = "windows")]
pub fn process_running(process_name: ProcessName) -> bool {
    let name = match process_name {
        ProcessName::P2pool => "p2pool",
        ProcessName::Xmrig => "xmrig",
        ProcessName::XmrigProxy => "xmrig-proxy",
        ProcessName::Node => "monerod",
        ProcessName::Xvb => panic!("XvB does not exist as a process outside of Gupaxx"),
    };
    let s = System::new_all();
    if s.processes_by_name(name.as_ref()).next().is_some() {
        return true;
    }
    false
}

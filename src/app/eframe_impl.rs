use std::sync::{Arc, Mutex};

use super::App;
#[cfg(target_os = "windows")]
use crate::errors::{process_running, ErrorButtons, ErrorFerris};
#[cfg(target_os = "windows")]
use crate::helper::ProcessName;
use crate::helper::{Helper, ProcessName, ProcessState};
use crate::inits::init_text_styles;
use crate::{NODE_MIDDLE, P2POOL_MIDDLE, SECOND, XMRIG_MIDDLE, XMRIG_PROXY_MIDDLE, XVB_MIDDLE};
use derive_more::derive::{Deref, DerefMut};
use log::debug;

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // *-------*
        // | DEBUG |
        // *-------*
        debug!("App | ----------- Start of [update()] -----------");
        // If closing
        self.quit(ctx);
        // Handle Keys
        let (key, wants_input) = self.keys_handle(ctx);

        // Refresh AT LEAST once a second
        debug!("App | Refreshing frame once per second");
        ctx.request_repaint_after(SECOND);

        // Get P2Pool/XMRig process state.
        // These values are checked multiple times so
        // might as well check only once here to save
        // on a bunch of [.lock().unwrap()]s.
        let mut process_states = ProcessStatesGui::new(self);
        // resize window and fonts if button "set" has been clicked in Gupaxx tab
        if self.must_resize {
            init_text_styles(ctx, self.state.gupax.selected_scale);
            self.must_resize = false;
        }
        // check for windows that a local instance of xmrig is not running outside of Gupaxx. Important because it could lead to crashes on this platform.
        // Warn only once per restart of Gupaxx.
        #[cfg(target_os = "windows")]
        if !self.xmrig_outside_warning_acknowledge
            && process_running(ProcessName::Xmrig)
            && !xmrig_is_alive
        {
            self.error_state.set("An instance of xmrig is running outside of Gupaxx.\nThis is not supported and could lead to crashes on this platform.\nPlease stop your local instance and start xmrig from Gupaxx Xmrig tab.", ErrorFerris::Error, ErrorButtons::Okay);
            self.xmrig_outside_warning_acknowledge = true;
        }
        // If there's an error, display [ErrorState] on the whole screen until user responds
        debug!("App | Checking if there is an error in [ErrorState]");
        if self.error_state.error {
            self.quit_error_panel(ctx, &process_states, &key);
            return;
        }
        // Compare [og == state] & [node_vec/pool_vec] and enable diff if found.
        // The struct fields are compared directly because [Version]
        // contains Arc<Mutex>'s that cannot be compared easily.
        // They don't need to be compared anyway.
        debug!("App | Checking diff between [og] & [state]");
        let og = self.og.lock().unwrap();
        self.diff = og.status != self.state.status
            || og.gupax != self.state.gupax
            || og.node != self.state.node
            || og.p2pool != self.state.p2pool
            || og.xmrig != self.state.xmrig
            || og.xmrig_proxy != self.state.xmrig_proxy
            || og.xvb != self.state.xvb
            || self.og_node_vec != self.node_vec
            || self.og_pool_vec != self.pool_vec;
        drop(og);

        self.top_panel(ctx);
        self.bottom_panel(ctx, &key, wants_input, &process_states);
        // xvb_is_alive is not the same for bottom and for middle.
        // for status we don't want to enable the column when it is retrying requests.
        // but also we don't want the user to be able to start it in this case.
        let p_xvb = process_states.find_mut(ProcessName::Xvb);
        p_xvb.alive = p_xvb.state != ProcessState::Dead;
        self.middle_panel(ctx, frame, key, &process_states);
    }
}
#[derive(Debug)]
pub struct ProcessStateGui {
    pub name: ProcessName,
    pub state: ProcessState,
    pub alive: bool,
    pub waiting: bool,
}

impl ProcessStateGui {
    pub fn run_middle_msg(&self) -> &str {
        match self.name {
            ProcessName::Node => NODE_MIDDLE,
            ProcessName::P2pool => P2POOL_MIDDLE,
            ProcessName::Xmrig => XMRIG_MIDDLE,
            ProcessName::XmrigProxy => XMRIG_PROXY_MIDDLE,
            ProcessName::Xvb => XVB_MIDDLE,
        }
    }
    pub fn stop(&self, helper: &Arc<Mutex<Helper>>) {
        match self.name {
            ProcessName::Node => Helper::stop_node(helper),
            ProcessName::P2pool => Helper::stop_p2pool(helper),
            ProcessName::Xmrig => Helper::stop_xmrig(helper),
            ProcessName::XmrigProxy => Helper::stop_xp(helper),
            ProcessName::Xvb => Helper::stop_xvb(helper),
        }
    }
}

#[derive(Deref, DerefMut, Debug)]
pub struct ProcessStatesGui(Vec<ProcessStateGui>);

impl ProcessStatesGui {
    // order is important for lock
    pub fn new(app: &App) -> Self {
        let mut process_states = ProcessStatesGui(vec![]);
        for process in [
            &app.node,
            &app.p2pool,
            &app.xmrig,
            &app.xmrig_proxy,
            &app.xvb,
        ] {
            let lock = process.lock().unwrap();
            process_states.push(ProcessStateGui {
                name: lock.name,
                alive: lock.is_alive(),
                waiting: lock.is_waiting(),
                state: lock.state,
            });
        }
        process_states
    }
    pub fn is_alive(&self, name: ProcessName) -> bool {
        self.iter()
            .find(|p| p.name == name)
            .unwrap_or_else(|| panic!("This vec should always contains all Processes {:?}", self))
            .alive
    }
    pub fn find(&self, name: ProcessName) -> &ProcessStateGui {
        self.iter()
            .find(|p| p.name == name)
            .unwrap_or_else(|| panic!("This vec should always contains all Processes {:?}", self))
    }
    pub fn find_mut(&mut self, name: ProcessName) -> &mut ProcessStateGui {
        self.iter_mut()
            .find(|p| p.name == name)
            .expect("This vec should always contains all Processes")
    }
}

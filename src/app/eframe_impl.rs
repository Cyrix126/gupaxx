use super::App;
use crate::helper::ProcessState;
use crate::macros::lock;
use crate::SECOND;
use egui::CentralPanel;
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
        debug!("App | Locking and collecting P2Pool state...");
        let p2pool = lock!(self.p2pool);
        let p2pool_is_alive = p2pool.is_alive();
        let p2pool_is_waiting = p2pool.is_waiting();
        let p2pool_state = p2pool.state;
        drop(p2pool);
        debug!("App | Locking and collecting XMRig state...");
        let xmrig = lock!(self.xmrig);
        let xmrig_is_alive = xmrig.is_alive();
        let xmrig_is_waiting = xmrig.is_waiting();
        let xmrig_state = xmrig.state;
        drop(xmrig);
        debug!("App | Locking and collecting XMRig-Proxy state...");
        let xmrig_proxy = lock!(self.xmrig_proxy);
        let xmrig_proxy_is_alive = xmrig_proxy.is_alive();
        let xmrig_proxy_is_waiting = xmrig_proxy.is_waiting();
        let xmrig_proxy_state = xmrig_proxy.state;
        drop(xmrig_proxy);
        debug!("App | Locking and collecting XvB state...");
        let xvb = lock!(self.xvb);
        let xvb_is_alive = xvb.is_alive();
        let xvb_is_waiting = xvb.is_waiting();
        let xvb_state = xvb.state;
        drop(xvb);

        // This sets the top level Ui dimensions.
        // Used as a reference for other uis.
        debug!("App | Setting width/height");
        CentralPanel::default().show(ctx, |ui| {
            let available_width = ui.available_width();
            if self.size.x != available_width {
                self.size.x = available_width;
                if self.now.elapsed().as_secs() > 5 {
                    self.must_resize = true;
                }
            };
            self.size.y = ui.available_height();
        });
        self.resize(ctx);

        // If there's an error, display [ErrorState] on the whole screen until user responds
        debug!("App | Checking if there is an error in [ErrorState]");
        if self.error_state.error {
            self.quit_error_panel(ctx, p2pool_is_alive, xmrig_is_alive, &key);
            return;
        }
        // Compare [og == state] & [node_vec/pool_vec] and enable diff if found.
        // The struct fields are compared directly because [Version]
        // contains Arc<Mutex>'s that cannot be compared easily.
        // They don't need to be compared anyway.
        debug!("App | Checking diff between [og] & [state]");
        let og = lock!(self.og);
        self.diff = og.status != self.state.status
            || og.gupax != self.state.gupax
            || og.p2pool != self.state.p2pool
            || og.xmrig != self.state.xmrig
            || og.xvb != self.state.xvb
            || self.og_node_vec != self.node_vec
            || self.og_pool_vec != self.pool_vec;
        drop(og);

        self.top_panel(ctx);
        self.bottom_panel(
            ctx,
            p2pool_state,
            xmrig_state,
            xmrig_proxy_state,
            xvb_state,
            &key,
            wants_input,
            p2pool_is_waiting,
            xmrig_is_waiting,
            xmrig_proxy_is_waiting,
            xvb_is_waiting,
            p2pool_is_alive,
            xmrig_is_alive,
            xmrig_proxy_is_alive,
            xvb_is_alive,
        );
        // xvb_is_alive is not the same for bottom and for middle.
        // for status we don't want to enable the column when it is retrying request
        // but for bottom we don't want the user to be able to start it in this case.
        let xvb_is_alive = xvb_state != ProcessState::Retry && xvb_state != ProcessState::Dead;
        self.middle_panel(
            ctx,
            frame,
            key,
            p2pool_is_alive,
            xmrig_is_alive,
            xmrig_proxy_is_alive,
            xvb_is_alive,
        );
    }
}

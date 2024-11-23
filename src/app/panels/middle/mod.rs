use crate::app::Tab;
use crate::app::eframe_impl::ProcessStatesGui;
use crate::app::keys::KeyPressed;
use crate::helper::ProcessName;
use crate::utils::constants::*;
use egui::*;
use log::debug;
mod about;
mod gupax;
mod node;
mod p2pool;
mod status;
mod xmrig;
mod xmrig_proxy;
mod xvb;
impl crate::app::App {
    #[allow(clippy::too_many_arguments)]
    pub fn middle_panel(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
        key: KeyPressed,
        states: &ProcessStatesGui,
    ) {
        // Middle panel, contents of the [Tab]
        debug!("App | Rendering CENTRAL_PANEL (tab contents)");
        CentralPanel::default().show(ctx, |ui| {
            self.size.x = ui.available_width();
            self.size.y = ui.available_height();
            // This sets the Ui dimensions after Top/Bottom are filled
            ui.style_mut().override_text_style = Some(TextStyle::Body);
            match self.tab {
                Tab::About => self.about_show(key, ui),
                Tab::Status => {
                    debug!("App | Entering [Status] Tab");
                    crate::disk::state::Status::show(
                        &mut self.state.status,
                        &self.pub_sys,
                        &self.node_api,
                        &self.p2pool_api,
                        &self.xmrig_api,
                        &self.xmrig_proxy_api,
                        &self.xvb_api,
                        &self.p2pool_img,
                        &self.xmrig_img,
                        states,
                        self.max_threads,
                        &self.gupax_p2pool_api,
                        &self.benchmarks,
                        self.size,
                        ctx,
                        ui,
                    );
                }
                Tab::Gupax => {
                    debug!("App | Entering [Gupax] Tab");
                    crate::disk::state::Gupax::show(
                        &mut self.state.gupax,
                        &self.og,
                        &self.state_path,
                        &self.update,
                        &self.file_window,
                        &mut self.error_state,
                        &self.restart,
                        self.size,
                        frame,
                        ctx,
                        ui,
                        &mut self.must_resize,
                    );
                }
                Tab::Node => {
                    debug!("App | Entering [Node] Tab");
                    crate::disk::state::Node::show(
                        &mut self.state.node,
                        &self.node,
                        &self.node_api,
                        &mut self.node_stdin,
                        self.size,
                        &self.file_window,
                        ui,
                    );
                }
                Tab::P2pool => {
                    debug!("App | Entering [P2Pool] Tab");
                    crate::disk::state::P2pool::show(
                        &mut self.state.p2pool,
                        &mut self.node_vec,
                        &self.og,
                        &self.ping,
                        &self.p2pool,
                        &self.p2pool_api,
                        &mut self.p2pool_stdin,
                        self.size,
                        ctx,
                        ui,
                    );
                }
                Tab::Xmrig => {
                    debug!("App | Entering [XMRig] Tab");
                    crate::disk::state::Xmrig::show(
                        &mut self.state.xmrig,
                        &mut self.pool_vec,
                        &self.xmrig,
                        &self.xmrig_api,
                        &mut self.xmrig_stdin,
                        self.size,
                        ctx,
                        ui,
                    );
                }
                Tab::XmrigProxy => {
                    debug!("App | Entering [XMRig-Proxy] Tab");
                    crate::disk::state::XmrigProxy::show(
                        &mut self.state.xmrig_proxy,
                        &self.xmrig_proxy,
                        &mut self.pool_vec,
                        &self.xmrig_proxy_api,
                        &mut self.xmrig_proxy_stdin,
                        self.size,
                        ui,
                    );
                }
                Tab::Xvb => {
                    debug!("App | Entering [XvB] Tab");
                    crate::disk::state::Xvb::show(
                        &mut self.state.xvb,
                        self.size,
                        &self.state.p2pool.address,
                        ctx,
                        ui,
                        &self.xvb_api,
                        &self.xmrig_api,
                        &self.xmrig_proxy_api,
                        states.is_alive(ProcessName::Xvb),
                    );
                }
            }
        });
    }
}

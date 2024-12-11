use egui::{Button, ComboBox, RichText, SelectableLabel, TextStyle, Ui};
use log::{debug, info};

use crate::{
    LIST_ADD, LIST_CLEAR, LIST_DELETE, LIST_SAVE,
    disk::{node::Node, pool::Pool, state::SelectedPoolNode},
};
#[derive(Clone, Debug, PartialEq)]
pub enum PoolNode {
    Node(Node),
    Pool(Pool),
}

impl PoolNode {
    pub fn ip(&self) -> &str {
        match &self {
            PoolNode::Node(n) => &n.ip,
            PoolNode::Pool(p) => &p.ip,
        }
    }
    pub fn port(&self) -> &str {
        match &self {
            PoolNode::Node(n) => &n.rpc,
            PoolNode::Pool(p) => &p.port,
        }
    }
    pub fn custom(&self) -> &str {
        match &self {
            PoolNode::Node(n) => &n.zmq,
            PoolNode::Pool(p) => &p.rig,
        }
    }
    pub fn custom_name(&self) -> &str {
        match &self {
            PoolNode::Node(_) => "ZMQ",
            PoolNode::Pool(_) => "rig",
        }
    }
    fn set_ip(&mut self, new_ip: String) {
        match self {
            PoolNode::Node(n) => n.ip = new_ip,
            PoolNode::Pool(p) => p.ip = new_ip,
        }
    }
    fn set_port(&mut self, new_port: String) {
        match self {
            PoolNode::Node(n) => n.rpc = new_port,
            PoolNode::Pool(p) => p.port = new_port,
        }
    }
    fn set_custom(&mut self, new_custom: String) {
        match self {
            PoolNode::Node(n) => n.zmq = new_custom,
            PoolNode::Pool(p) => p.rig = new_custom,
        }
    }
}
/// compatible for P2Pool and Xmrig/Proxy
/// current is (name, ip, port, zmq/rig)
pub fn list_poolnode(
    ui: &mut Ui,
    current: &mut (&mut String, &mut String, &mut String, &mut String),
    selected: &mut SelectedPoolNode,
    node_vec: &mut Vec<(String, PoolNode)>,
    incorrect_input: bool,
) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let width = ui
            .available_width()
            .max(ui.text_style_height(&TextStyle::Button) * 28.0);
        // [Manual node selection]
        // [Ping List]
        debug!("P2Pool Tab | Rendering [Node List]");
        // [Menu]
        menu_list_node(ui, node_vec, width, selected, current);
        let node_vec_len = node_vec.len();
        // [Add/Save]
        ui.horizontal(|ui| {
            add_save_node(
                ui,
                selected,
                node_vec,
                current,
                node_vec_len,
                incorrect_input,
            );
        });
        // [Delete]
        ui.horizontal(|ui| {
            delete_node(ui, selected, node_vec, current, node_vec_len);
        });
        // [Clear]
        ui.horizontal(|ui| {
            clear_node(ui, current);
        });
    });
}
// slider H/s

fn clear_node(ui: &mut Ui, current: &mut (&mut String, &mut String, &mut String, &mut String)) {
    ui.add_enabled_ui(
        !current.0.is_empty()
            || !current.1.is_empty()
            || !current.2.is_empty()
            || !current.3.is_empty(),
        |ui| {
            if ui
                .add_sized([ui.available_width(), 0.0], Button::new("Clear"))
                .on_hover_text(LIST_CLEAR)
                .clicked()
            {
                current.0.clear();
                current.1.clear();
                current.2.clear();
                current.3.clear();
            }
        },
    );
}
fn menu_list_node(
    ui: &mut Ui,
    node_vec: &mut [(String, PoolNode)],
    width: f32,
    selected: &mut SelectedPoolNode,
    current: &mut (&mut String, &mut String, &mut String, &mut String),
) {
    let text = RichText::new(format!("{}. {}", selected.index + 1, selected.name));
    ComboBox::from_id_salt("manual_nodes")
        .selected_text(text)
        .width(width)
        .show_ui(ui, |ui| {
            for (n, (name, node)) in node_vec.iter().enumerate() {
                let text = RichText::new(format!(
                    "{}. {}\n     IP: {}\n    RPC: {}\n    {}: {}",
                    n + 1,
                    name,
                    node.ip(),
                    node.port(),
                    node.custom_name(),
                    node.custom()
                ));
                if ui
                    .add(SelectableLabel::new(selected.name == **name, text))
                    .clicked()
                {
                    selected.index = n;
                    let node = node.clone();
                    selected.name.clone_from(name);
                    selected.ip.clone_from(&node.ip().to_string());
                    selected.rpc.clone_from(&node.port().to_string());
                    selected.zmq_rig.clone_from(&node.custom().to_string());
                    current.0.clone_from(name);
                    *current.1 = node.ip().to_string();
                    *current.2 = node.port().to_string();
                    *current.3 = node.custom().to_string();
                }
            }
        });
}
fn add_save_node(
    ui: &mut Ui,
    selected: &mut SelectedPoolNode,
    node_vec: &mut Vec<(String, PoolNode)>,
    current: &mut (&mut String, &mut String, &mut String, &mut String),
    node_vec_len: usize,
    incorrect_input: bool,
) {
    // list should never be empty unless state edited by hand.
    let is_node = matches!(node_vec[0].1, PoolNode::Node(_));
    // [Add/Save]
    let mut exists = false;
    let mut save_diff = true;
    let mut existing_index = 0;
    for (name, node) in node_vec.iter() {
        if *name == *current.0 {
            exists = true;
            if *current.1 == node.ip() && *current.2 == node.port() && *current.3 == node.custom() {
                save_diff = false;
            }
            break;
        }
        existing_index += 1;
    }
    let text = if exists { LIST_SAVE } else { LIST_ADD };
    let text = format!(
        "{}\n    Currently selected node: {}. {}\n    Current amount of {}: {}/1000",
        text,
        selected.index + 1,
        selected.name,
        if is_node { "nodes" } else { "pools" },
        node_vec_len
    );
    // If the node already exists, show [Save] and mutate the already existing node
    if exists {
        ui.add_enabled_ui(!incorrect_input && save_diff, |ui| {
            if ui
                .add_sized([ui.available_width(), 0.0], Button::new("Save"))
                .on_hover_text(text)
                .clicked()
            {
                let ip = current.1.clone();
                let rpc = current.2.clone();
                // zmq can be rig in case of Pool
                let zmq = current.3.clone();
                let poolnode = &mut node_vec[existing_index].1;
                poolnode.set_ip(ip);
                poolnode.set_port(rpc);
                poolnode.set_custom(zmq);
                info!(
                    "Node | S | [index: {}, name: \"{}\", ip: \"{}\", rpc: {}, {}: {}]",
                    existing_index + 1,
                    current.0,
                    current.1,
                    current.2,
                    poolnode.custom_name(),
                    current.3
                );
                selected.index = existing_index;
                selected.ip.clone_from(current.1);
                selected.rpc.clone_from(current.2);
                selected.zmq_rig.clone_from(current.3);
            }
        });
    // Else, add to the list
    } else {
        ui.add_enabled_ui(!incorrect_input && node_vec_len < 1000, |ui| {
            if ui
                .add_sized([ui.available_width(), 0.0], Button::new("Add"))
                .on_hover_text(text)
                .clicked()
            {
                let ip = current.1.clone();
                let rpc = current.2.clone();
                // zmq can be rig in case of Pool
                let zmq = current.3.clone();
                let poolnode = match node_vec[existing_index].1 {
                    PoolNode::Node(_) => PoolNode::Node(Node { ip, rpc, zmq }),
                    PoolNode::Pool(_) => PoolNode::Pool(Pool {
                        rig: zmq,
                        ip,
                        port: rpc,
                    }),
                };
                info!(
                    "Node | A | [index: {}, name: \"{}\", ip: \"{}\", rpc: {}, {}: {}]",
                    node_vec_len,
                    current.0,
                    current.1,
                    current.2,
                    poolnode.custom_name(),
                    current.3
                );
                node_vec.push((current.0.clone(), poolnode));
                selected.index = node_vec_len;
                selected.name.clone_from(current.0);
                selected.ip.clone_from(current.1);
                selected.rpc.clone_from(current.2);
                selected.zmq_rig.clone_from(current.3);
            }
        });
    }
}

fn delete_node(
    ui: &mut Ui,
    selected: &mut SelectedPoolNode,
    node_vec: &mut Vec<(String, PoolNode)>,
    current: &mut (&mut String, &mut String, &mut String, &mut String),
    node_vec_len: usize,
) {
    ui.add_enabled_ui(node_vec_len > 1, |ui| {
        let text = format!(
            "{}\n    Currently selected node: {}. {}\n    Current amount of nodes: {}/1000",
            LIST_DELETE,
            selected.index + 1,
            selected.name,
            node_vec_len
        );
        if ui
            .add_sized([ui.available_width(), 0.0], Button::new("Delete"))
            .on_hover_text(text)
            .clicked()
        {
            let new_name;
            let new_node;
            match selected.index {
                0 => {
                    new_name = node_vec[1].0.clone();
                    new_node = node_vec[1].1.clone();
                    node_vec.remove(0);
                }
                _ => {
                    node_vec.remove(selected.index);
                    selected.index -= 1;
                    new_name = node_vec[selected.index].0.clone();
                    new_node = node_vec[selected.index].1.clone();
                }
            };
            selected.name.clone_from(&new_name);
            selected.ip = new_node.ip().to_string();
            selected.rpc = new_node.port().to_string();
            selected.zmq_rig = new_node.custom().to_string();
            *current.0 = new_name;
            *current.1 = new_node.ip().to_string();
            *current.2 = new_node.port().to_string();
            *current.3 = new_node.custom().to_string();
            info!(
                "Node | D | [index: {}, name: \"{}\", ip: \"{}\", port: {}, {}: {}]",
                selected.index,
                selected.name,
                selected.ip,
                selected.rpc,
                new_node.custom_name(),
                selected.zmq_rig
            );
        }
    });
}

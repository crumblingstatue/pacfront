use {
    super::{Tab, tabs::package::PkgTab},
    crate::app::PacfrontApp,
    eframe::egui,
    egui_dock::{Node, NodeIndex, TabIndex},
};

#[derive(Default)]
pub struct CmdBuf {
    cmds: Vec<Cmd>,
}

impl CmdBuf {
    pub fn push(&mut self, cmd: Cmd) {
        self.cmds.push(cmd);
    }
}

pub enum Cmd {
    OpenPkgTab { name: String, remote: bool },
}

pub fn process_cmds(app: &mut PacfrontApp, _ctx: &egui::Context) {
    for cmd in std::mem::take(&mut app.ui.shared.cmd.cmds) {
        match cmd {
            Cmd::OpenPkgTab { name, remote } => {
                // First, try to activate already existing tab for this package
                let mut focus_indices = None;
                for (node_idx, (surf_idx, node)) in
                    app.ui.dock_state.iter_all_nodes_mut().enumerate()
                {
                    if let Node::Leaf { tabs, active, .. } = node {
                        for (tab_idx, tab) in tabs.iter_mut().enumerate() {
                            if let Tab::LocalPkg(pkg_tab) = tab
                                && pkg_tab.name == name
                            {
                                focus_indices = Some((surf_idx, NodeIndex(node_idx)));
                                *active = TabIndex(tab_idx);
                            }
                        }
                    }
                }
                // FIXME: Really awkward code to try to not open package tab on top of package list tab, if
                // there is another tab group (node) already open with packages.
                if let Some(indices) = focus_indices {
                    app.ui.dock_state.set_focused_node_and_surface(indices);
                } else {
                    for node in app.ui.dock_state.main_surface_mut().iter_mut() {
                        if let Node::Leaf { tabs, active, .. } = node {
                            if tabs.iter().any(|tab| {
                                std::mem::discriminant(tab)
                                    == std::mem::discriminant(&Tab::LocalPkgList)
                            }) {
                                continue;
                            }
                            if remote {
                                tabs.push(Tab::RemotePkg(PkgTab::new(name)));
                            } else {
                                tabs.push(Tab::LocalPkg(PkgTab::new(name)));
                            }
                            *active = TabIndex(tabs.len().saturating_sub(1));
                            return;
                        }
                    }
                    let pkg = if remote {
                        Tab::RemotePkg(PkgTab::new(name))
                    } else {
                        Tab::LocalPkg(PkgTab::new(name))
                    };
                    app.ui.dock_state.push_to_first_leaf(pkg);
                }
            }
        }
    }
}

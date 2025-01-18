use {
    super::{Tab, tabs::package::PkgTab},
    crate::{alpm_util::PkgId, app::PacfrontApp},
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
    OpenPkgTab(PkgId),
}

pub fn process_cmds(app: &mut PacfrontApp, _ctx: &egui::Context) {
    for cmd in std::mem::take(&mut app.ui.shared.cmd.cmds) {
        match cmd {
            Cmd::OpenPkgTab(id) => {
                // First, try to activate already existing tab for this package
                let mut focus_indices = None;
                for (node_idx, (surf_idx, node)) in
                    app.ui.dock_state.iter_all_nodes_mut().enumerate()
                {
                    if let Node::Leaf { tabs, active, .. } = node {
                        for (tab_idx, tab) in tabs.iter_mut().enumerate() {
                            if let Tab::Pkg(pkg_tab) = tab
                                && pkg_tab.id == id
                            {
                                focus_indices = Some((surf_idx, NodeIndex(node_idx)));
                                *active = TabIndex(tab_idx);
                            }
                        }
                    }
                }
                // Determine where to place the new tab.
                //
                // For now, we just push to the last leaf node, and hope that's good enough.
                #[expect(clippy::collapsible_else_if)]
                if let Some(indices) = focus_indices {
                    app.ui.dock_state.set_focused_node_and_surface(indices);
                } else {
                    if let Some(Node::Leaf { tabs, active, .. }) = app
                        .ui
                        .dock_state
                        .main_surface_mut()
                        .iter_mut()
                        .rfind(|node| node.is_leaf())
                    {
                        tabs.push(Tab::Pkg(PkgTab::new(id)));
                        *active = TabIndex(tabs.len().saturating_sub(1));
                    } else {
                        app.ui
                            .dock_state
                            .push_to_first_leaf(Tab::Pkg(PkgTab::new(id)));
                    }
                }
            }
        }
    }
}

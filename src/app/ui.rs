use {
    super::{PacfrontApp, ouroboros_impl_pac_state::PacState},
    eframe::egui,
    egui_dock::{DockArea, DockState, Node, NodeIndex, TabIndex, TabViewer},
    egui_extras::{Column, TableBuilder},
    tabs::package::PkgTab,
};

mod tabs {
    pub mod package;
}

pub(super) struct UiState {
    dock_state: DockState<Tab>,
    shared: SharedUiState,
}

#[derive(Default)]
struct SharedUiState {
    filter_string: String,
    cmd: CmdBuf,
}

#[derive(Default)]
struct CmdBuf {
    cmds: Vec<Cmd>,
}

impl CmdBuf {
    fn push(&mut self, cmd: Cmd) {
        self.cmds.push(cmd);
    }
}

enum Cmd {
    OpenPkgTab { name: String, remote: bool },
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            shared: Default::default(),
            dock_state: DockState::new(vec![Tab::LocalDb, Tab::SyncDbList, Tab::SyncDbPkgList]),
        }
    }
}

struct TabViewState<'pac, 'ui> {
    pac: &'pac mut PacState,
    ui: &'ui mut SharedUiState,
}

impl TabViewer for TabViewState<'_, '_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tab::LocalDb => format!("Local packages ({})", self.pac.borrow_pkg_list().len()).into(),
            Tab::SyncDbPkgList => format!(
                "Remote packages ({})",
                self.pac
                    .borrow_sync()
                    .iter()
                    .map(|db| db.pkgs().len())
                    .sum::<usize>()
            )
            .into(),
            Tab::LocalPkg(pkg) => format!("Package '{}'", pkg.name).into(),
            Tab::RemotePkg(pkg) => format!("Remote Package '{}'", pkg.name).into(),
            Tab::SyncDbList => format!("Sync DBs ({})", self.pac.borrow_sync().len()).into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::LocalDb => package_list_ui(ui, self.pac, self.ui),
            Tab::SyncDbPkgList => sync_package_list_ui(ui, self.pac, self.ui),
            Tab::LocalPkg(tab) => tabs::package::ui(ui, self.pac, self.ui, tab, false),
            Tab::RemotePkg(tab) => tabs::package::ui(ui, self.pac, self.ui, tab, true),
            Tab::SyncDbList => syncdb_list_ui(ui, self.pac, self.ui),
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        #[expect(clippy::match_like_matches_macro)]
        match tab {
            Tab::LocalDb => false,
            _ => true,
        }
    }

    fn force_close(&mut self, tab: &mut Self::Tab) -> bool {
        match tab {
            Tab::LocalDb => false,
            Tab::LocalPkg(pkg_tab) => pkg_tab.force_close,
            Tab::RemotePkg(pkg_tab) => pkg_tab.force_close,
            Tab::SyncDbList => false,
            Tab::SyncDbPkgList => false,
        }
    }
}

fn package_list_ui(ui: &mut egui::Ui, pac: &mut PacState, ui_state: &mut SharedUiState) {
    egui::TopBottomPanel::top("top_panel").show_inside(ui, |ui| {
        ui.horizontal(|ui| {
            pac.with_mut(|this| {
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut ui_state.filter_string)
                            .hint_text("🔍 Filter"),
                    )
                    .changed()
                {
                    *this.filtered_local_pkgs = this
                        .pkg_list
                        .iter()
                        .filter(|pkg| {
                            let filt_lo = ui_state.filter_string.to_ascii_lowercase();
                            pkg.name().contains(&filt_lo)
                                || pkg.desc().is_some_and(|desc| {
                                    desc.to_ascii_lowercase().contains(&filt_lo)
                                })
                                || pkg
                                    .provides()
                                    .iter()
                                    .any(|dep| dep.name().contains(&filt_lo))
                        })
                        .copied()
                        .collect();
                }
                ui.spacing();
                ui.label(format!(
                    "{} packages listed",
                    this.filtered_local_pkgs.len()
                ));
            });
        });
    });
    TableBuilder::new(ui)
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::remainder())
        .auto_shrink(false)
        .striped(true)
        .header(32.0, |mut row| {
            row.col(|ui| {
                ui.label("Name");
            });
            row.col(|ui| {
                ui.label("Version");
            });
            row.col(|ui| {
                ui.label("Description");
            });
        })
        .body(|mut body| {
            body.ui_mut().style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            pac.with_filtered_local_pkgs(|list| {
                body.rows(24.0, list.len(), |mut row| {
                    let pkg = &list[row.index()];
                    row.col(|ui| {
                        if ui.link(pkg.name()).clicked() {
                            ui_state.cmd.push(Cmd::OpenPkgTab {
                                name: pkg.name().to_string(),
                                remote: false,
                            });
                        }
                    });
                    row.col(|ui| {
                        ui.label(pkg.version().to_string());
                    });
                    row.col(|ui| {
                        ui.label(pkg.desc().unwrap_or("<missing description>"));
                    });
                });
            });
        });
}

fn sync_package_list_ui(ui: &mut egui::Ui, pac: &mut PacState, ui_state: &mut SharedUiState) {
    egui::TopBottomPanel::top("top_panel").show_inside(ui, |ui| {
        ui.horizontal(|ui| {
            pac.with_mut(|this| {
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut ui_state.filter_string)
                            .hint_text("🔍 Filter"),
                    )
                    .changed()
                {
                    *this.filtered_sync_pkgs = this
                        .sync_pkg_list
                        .iter()
                        .filter(|pkg| {
                            let filt_lo = ui_state.filter_string.to_ascii_lowercase();
                            pkg.name().contains(&filt_lo)
                                || pkg.desc().is_some_and(|desc| {
                                    desc.to_ascii_lowercase().contains(&filt_lo)
                                })
                        })
                        .copied()
                        .collect();
                }
                ui.spacing();
                ui.label(format!("{} packages listed", this.filtered_sync_pkgs.len()));
            });
        });
    });
    TableBuilder::new(ui)
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::remainder())
        .auto_shrink(false)
        .striped(true)
        .header(32.0, |mut row| {
            row.col(|ui| {
                ui.label("Name");
            });
            row.col(|ui| {
                ui.label("Version");
            });
            row.col(|ui| {
                ui.label("Description");
            });
        })
        .body(|mut body| {
            body.ui_mut().style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            pac.with_mut(|this| {
                let list = this.filtered_sync_pkgs;
                body.rows(24.0, list.len(), |mut row| {
                    let pkg = &list[row.index()];
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            let dbname = pkg.db().map_or("<missing db>", |db| db.name());
                            if ui.link(format!("{}/{}", dbname, pkg.name())).clicked() {
                                ui_state.cmd.push(Cmd::OpenPkgTab {
                                    name: pkg.name().to_string(),
                                    remote: true,
                                });
                            }
                            if this.pkg_list.iter().any(|pkg2| pkg2.name() == pkg.name()) {
                                ui.label("[installed]");
                            }
                        });
                    });
                    row.col(|ui| {
                        ui.label(pkg.version().to_string());
                    });
                    row.col(|ui| {
                        ui.label(pkg.desc().unwrap_or("<missing description>"));
                    });
                });
            });
        });
}

#[derive(Default)]
pub enum Tab {
    #[default]
    LocalDb,
    SyncDbList,
    SyncDbPkgList,
    LocalPkg(PkgTab),
    RemotePkg(PkgTab),
}

pub fn top_panel_ui(_app: &mut PacfrontApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.label("📦 Pacfront");
    });
}

pub fn central_panel_ui(app: &mut PacfrontApp, ctx: &egui::Context) {
    DockArea::new(&mut app.ui.dock_state)
        .show_leaf_collapse_buttons(false)
        .show_leaf_close_all_buttons(false)
        .show(ctx, &mut TabViewState {
            pac: &mut app.pac,
            ui: &mut app.ui.shared,
        });
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
                                std::mem::discriminant(tab) == std::mem::discriminant(&Tab::LocalDb)
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

fn syncdb_list_ui(ui: &mut egui::Ui, pac: &mut PacState, _ui_state: &mut SharedUiState) {
    pac.with_sync_mut(|sync| {
        for db in sync {
            ui.label(format!("{} ({})", db.name(), db.pkgs().len()));
        }
    });
}

use {
    super::{PacfrontApp, ouroboros_impl_pac_state::PacState},
    eframe::egui,
    egui_dock::{DockArea, DockState, Node, NodeIndex, TabIndex, TabViewer},
    egui_extras::{Column, TableBuilder},
    std::{path::Path, process::Command},
};

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
            Tab::LocalPkg(tab) => package_ui(ui, self.pac, self.ui, tab, false),
            Tab::RemotePkg(tab) => package_ui(ui, self.pac, self.ui, tab, true),
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
            pac.with_filtered_sync_pkgs(|list| {
                body.rows(24.0, list.len(), |mut row| {
                    let pkg = &list[row.index()];
                    row.col(|ui| {
                        let dbname = pkg.db().map_or("<missing db>", |db| db.name());
                        if ui.link(format!("{}/{}", dbname, pkg.name())).clicked() {
                            ui_state.cmd.push(Cmd::OpenPkgTab {
                                name: pkg.name().to_string(),
                                remote: true,
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

fn package_ui(
    ui: &mut egui::Ui,
    pac: &PacState,
    ui_state: &mut SharedUiState,
    pkg_tab: &mut PkgTab,
    remote: bool,
) {
    if ui.input(|inp| {
        let esc = inp.key_pressed(egui::Key::Escape);
        let ctrl_w = inp.modifiers.ctrl && inp.key_pressed(egui::Key::W);
        esc || ctrl_w
    }) {
        pkg_tab.force_close = true;
    }
    pac.with(|this| {
        let pkg_list = if remote {
            this.sync_pkg_list
        } else {
            this.pkg_list
        };
        match pkg_list.iter().find(|pkg| pkg.name() == pkg_tab.name) {
            Some(pkg) => {
                ui.horizontal(|ui| {
                    if let Some(db) = pkg.db() {
                        ui.label(format!("{}/", db.name()));
                    }
                    ui.heading(pkg.name());
                    ui.label(pkg.version().to_string());
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut pkg_tab.tab, PkgTabTab::General, "General");
                    ui.selectable_value(&mut pkg_tab.tab, PkgTabTab::Files, "File list");
                });
                ui.separator();
                match pkg_tab.tab {
                    PkgTabTab::General => {
                        ui.label(pkg.desc().unwrap_or("<no description>"));
                        if let Some(url) = pkg.url() {
                            ui.horizontal(|ui| {
                                ui.label("URL");
                                ui.hyperlink(url);
                            });
                        }
                        let deps = pkg.depends();
                        ui.heading(format!("Dependencies ({})", deps.len()));
                        if deps.is_empty() {
                            ui.label("<none>");
                        } else {
                            ui.horizontal_wrapped(|ui| {
                                for dep in deps {
                                    let resolved = pkg_list.iter().find(|pkg| {
                                        pkg.name() == dep.name()
                                            || pkg
                                                .provides()
                                                .iter()
                                                .any(|dep2| dep2.name() == dep.name())
                                    });
                                    match resolved {
                                        Some(pkg) => {
                                            let label = if dep.name() == pkg.name() {
                                                dep.name()
                                            } else {
                                                &format!("{} ({})", dep.name(), pkg.name())
                                            };
                                            if ui.link(label).clicked() {
                                                ui_state.cmd.push(Cmd::OpenPkgTab {
                                                    name: pkg.name().to_string(),
                                                    remote,
                                                });
                                            }
                                        }
                                        None => {
                                            ui.label(format!("{} (unresolved)", dep));
                                        }
                                    }
                                }
                            });
                        }
                        let deps = pkg.optdepends();
                        ui.heading(format!("Optional dependencies ({})", deps.len()));
                        if deps.is_empty() {
                            ui.label("<none>");
                        } else {
                            for dep in deps {
                                ui.horizontal(|ui| {
                                    if ui.link(dep.name()).clicked() {
                                        ui_state.cmd.push(Cmd::OpenPkgTab {
                                            name: dep.name().to_string(),
                                            remote,
                                        });

                                        if let Some(ver) = dep.version() {
                                            ui.label(format!("={ver}"));
                                        }
                                    }
                                    if let Some(desc) = dep.desc() {
                                        ui.label(desc);
                                    }
                                });
                            }
                        }
                        let reqs = pkg.required_by();
                        ui.heading(format!("Required by ({})", reqs.len()));
                        if reqs.is_empty() {
                            ui.label("<none>");
                        } else {
                            ui.horizontal_wrapped(|ui| {
                                for req in reqs {
                                    if ui.link(&req).clicked() {
                                        ui_state.cmd.push(Cmd::OpenPkgTab { name: req, remote });
                                    }
                                }
                            });
                        }
                        let opt_for = pkg.optional_for();
                        ui.heading(format!("Optional for ({})", opt_for.len()));
                        if opt_for.is_empty() {
                            ui.label("<none>");
                        } else {
                            ui.horizontal_wrapped(|ui| {
                                for name in opt_for {
                                    if ui.link(&name).clicked() {
                                        ui_state.cmd.push(Cmd::OpenPkgTab { name, remote });
                                    }
                                }
                            });
                        }
                        let provides = pkg.provides();
                        ui.heading(format!("Provides ({})", provides.len()));
                        for dep in provides {
                            ui.label(dep.to_string());
                        }
                    }
                    PkgTabTab::Files => {
                        ui.add(
                            egui::TextEdit::singleline(&mut pkg_tab.files_filt_string)
                                .hint_text("🔍 Filter"),
                        );
                        let files = pkg.files();
                        let deduped_files = deduped_files(files.files()).filter(|file| {
                            file.name()
                                .to_ascii_lowercase()
                                .contains(&pkg_tab.files_filt_string.to_ascii_lowercase())
                        });
                        for file in deduped_files {
                            let name = format!("/{}", file.name());
                            if ui.link(&name).clicked() {
                                Command::new("xdg-open").arg(name).status().unwrap();
                            }
                        }
                    }
                }
            }
            None => {
                ui.label("<Unresolved package>");
            }
        }
    });
}

/// Filters out items from the package file list that are fully contained by the next item
/// (e.g. `/usr/bin`) is removed if the next item is `/usr/bin/cat`
fn deduped_files(list: &[alpm::File]) -> impl Iterator<Item = &alpm::File> {
    list.array_windows()
        .filter_map(|[a, b]| {
            let retain = !path_contains_other_path(b.name().as_ref(), a.name().as_ref());
            (retain).then_some(a)
        })
        .chain(list.last())
}

fn path_contains_other_path(haystack: &Path, needle: &Path) -> bool {
    haystack.parent() == Some(needle)
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

pub struct PkgTab {
    name: String,
    tab: PkgTabTab,
    force_close: bool,
    files_filt_string: String,
}

impl PkgTab {
    fn new(name: String) -> Self {
        Self {
            name,
            tab: PkgTabTab::default(),
            force_close: false,
            files_filt_string: String::new(),
        }
    }
}

#[derive(PartialEq, Default)]
enum PkgTabTab {
    #[default]
    General,
    Files,
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

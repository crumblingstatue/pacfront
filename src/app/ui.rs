use {
    super::{PacfrontApp, ouroboros_impl_pac_state::PacState},
    alpm::Package,
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
    OpenPkgTab(String),
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            shared: Default::default(),
            dock_state: DockState::new(vec![Tab::LocalDb]),
        }
    }
}

struct TabViewState<'pac, 'ui> {
    pac: &'pac PacState,
    ui: &'ui mut SharedUiState,
}

impl TabViewer for TabViewState<'_, '_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tab::LocalDb => "Local packages".into(),
            Tab::Package(pkg) => format!("Package '{}'", pkg.name).into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::LocalDb => package_list_ui(ui, self.pac, self.ui),
            Tab::Package(name) => package_ui(ui, self.pac, self.ui, name),
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
            Tab::Package(pkg_tab) => pkg_tab.force_close,
        }
    }
}

fn package_list_ui(ui: &mut egui::Ui, pac: &PacState, ui_state: &mut SharedUiState) {
    egui::TopBottomPanel::top("top_panel").show_inside(ui, |ui| {
        ui.add(egui::TextEdit::singleline(&mut ui_state.filter_string).hint_text("🔍 Filter"))
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
            pac.with_pkg_list(|list| {
                let filtered: Vec<&Package> = list
                    .iter()
                    .filter(|pkg| {
                        pkg.name().contains(&ui_state.filter_string)
                            || pkg
                                .desc()
                                .is_some_and(|desc| desc.contains(&ui_state.filter_string))
                    })
                    .copied()
                    .collect();
                body.rows(24.0, filtered.len(), |mut row| {
                    let pkg = &filtered[row.index()];
                    row.col(|ui| {
                        if ui.link(pkg.name()).clicked() {
                            ui_state.cmd.push(Cmd::OpenPkgTab(pkg.name().to_string()));
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
) {
    if ui.input(|inp| {
        let esc = inp.key_pressed(egui::Key::Escape);
        let ctrl_w = inp.modifiers.ctrl && inp.key_pressed(egui::Key::W);
        esc || ctrl_w
    }) {
        pkg_tab.force_close = true;
    }
    pac.with_pkg_list(
        |pkg_list| match pkg_list.iter().find(|pkg| pkg.name() == pkg_tab.name) {
            Some(pkg) => {
                ui.heading(pkg.name());
                ui.separator();
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut pkg_tab.tab, PkgTabTab::General, "General");
                    ui.selectable_value(&mut pkg_tab.tab, PkgTabTab::Files, "File list");
                });
                ui.separator();
                match pkg_tab.tab {
                    PkgTabTab::General => {
                        ui.label(pkg.desc().unwrap_or("<no description>"));
                        let deps = pkg.depends();
                        ui.heading("Dependencies");
                        if deps.is_empty() {
                            ui.label("<none>");
                        } else {
                            ui.horizontal_wrapped(|ui| {
                                for dep in deps {
                                    let lib_dep = dep.name().ends_with(".so");
                                    if lib_dep {
                                        ui.label(dep.to_string());
                                    } else if ui.link(dep.to_string()).clicked() {
                                        ui_state.cmd.push(Cmd::OpenPkgTab(dep.name().to_string()));
                                    }
                                }
                            });
                        }
                        ui.heading("Optional dependencies");
                        let deps = pkg.optdepends();
                        if deps.is_empty() {
                            ui.label("<none>");
                        } else {
                            for dep in deps {
                                ui.horizontal(|ui| {
                                    if ui.link(dep.name()).clicked() {
                                        ui_state.cmd.push(Cmd::OpenPkgTab(dep.name().to_string()));

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
        },
    );
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
    Package(PkgTab),
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
            pac: &app.pac,
            ui: &mut app.ui.shared,
        });
}

pub fn process_cmds(app: &mut PacfrontApp, _ctx: &egui::Context) {
    for cmd in std::mem::take(&mut app.ui.shared.cmd.cmds) {
        match cmd {
            Cmd::OpenPkgTab(name) => {
                // First, try to activate already existing tab for this package
                let mut focus_indices = None;
                for (node_idx, (surf_idx, node)) in
                    app.ui.dock_state.iter_all_nodes_mut().enumerate()
                {
                    if let Node::Leaf { tabs, active, .. } = node {
                        for (tab_idx, tab) in tabs.iter_mut().enumerate() {
                            if let Tab::Package(pkg_tab) = tab
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
                            tabs.push(Tab::Package(PkgTab::new(name)));
                            *active = TabIndex(tabs.len().saturating_sub(1));
                            return;
                        }
                    }
                    app.ui
                        .dock_state
                        .push_to_first_leaf(Tab::Package(PkgTab::new(name)));
                }
            }
        }
    }
}

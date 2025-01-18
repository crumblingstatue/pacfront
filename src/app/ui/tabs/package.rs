use {
    crate::{
        alpm_util::{PkgId, deduped_files},
        app::ui::{PacState, SharedUiState, cmd::Cmd},
    },
    eframe::egui,
    std::process::Command,
};

pub struct PkgTab {
    pub id: PkgId,
    tab: PkgTabTab,
    pub force_close: bool,
    files_filt_string: String,
}

impl PkgTab {
    pub fn new(id: PkgId) -> Self {
        Self {
            id,
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

pub fn ui(ui: &mut egui::Ui, pac: &PacState, ui_state: &mut SharedUiState, pkg_tab: &mut PkgTab) {
    if ui.input(|inp| {
        let esc = inp.key_pressed(egui::Key::Escape);
        let ctrl_w = inp.modifiers.ctrl && inp.key_pressed(egui::Key::W);
        esc || ctrl_w
    }) {
        pkg_tab.force_close = true;
    }
    let remote = pkg_tab.id.is_remote();
    pac.with(|this| {
        let pkg_list = if remote {
            this.sync_pkg_list
        } else {
            this.pkg_list
        };
        match pkg_list.iter().find(|pkg| pkg_tab.id.matches_pkg(pkg)) {
            Some(pkg) => {
                ui.horizontal(|ui| {
                    if let Some(db) = pkg.db() {
                        ui.label(format!("{}/", db.name()));
                    }
                    ui.heading(pkg.name());
                    ui.label(pkg.version().to_string());
                    if remote
                        && this.pkg_list.iter().any(|pkg2| pkg2.name() == pkg.name())
                        && ui.link("[installed]").clicked()
                    {
                        ui_state.cmd.push(Cmd::OpenPkgTab(PkgId::local(pkg.name())));
                    }
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
                                            || pkg.provides().iter().any(|dep2| {
                                                // TODO: This might not be correct/enough
                                                dep2.name() == dep.name()
                                                    && dep2.version() >= dep.version()
                                            })
                                    });
                                    match resolved {
                                        Some(pkg) => {
                                            let label = if dep.name() == pkg.name() {
                                                dep.name()
                                            } else {
                                                &format!("{} ({})", dep.name(), pkg.name())
                                            };
                                            if ui.link(label).clicked() {
                                                ui_state.cmd.push(Cmd::OpenPkgTab(
                                                    PkgId::qualified(&pkg_tab.id.db, pkg.name()),
                                                ));
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
                                        ui_state.cmd.push(Cmd::OpenPkgTab(PkgId::qualified(
                                            &pkg_tab.id.db,
                                            dep.name(),
                                        )));

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
                                        ui_state.cmd.push(Cmd::OpenPkgTab(PkgId::qualified(
                                            &pkg_tab.id.db,
                                            &req,
                                        )));
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
                                        ui_state.cmd.push(Cmd::OpenPkgTab(PkgId::qualified(
                                            &pkg_tab.id.db,
                                            &name,
                                        )));
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

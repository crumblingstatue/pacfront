use {
    crate::app::ui::{PacState, SharedUiState, cmd::Cmd},
    eframe::egui,
    egui_extras::{Column, TableBuilder},
};

pub fn ui(ui: &mut egui::Ui, pac: &mut PacState, ui_state: &mut SharedUiState) {
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
                            if this.pkg_list.iter().any(|pkg2| pkg2.name() == pkg.name())
                                && ui
                                    .add(
                                        egui::Label::new("[installed]").sense(egui::Sense::click()),
                                    )
                                    .clicked()
                            {
                                ui_state.cmd.push(Cmd::OpenPkgTab {
                                    name: pkg.name().to_string(),
                                    remote: false,
                                });
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

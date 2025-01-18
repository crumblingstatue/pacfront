use {
    super::PkgListState,
    crate::{
        alpm_util::PkgId,
        app::ui::{PacState, SharedUiState, cmd::Cmd},
    },
    alpm::Package,
    eframe::egui,
    egui_extras::{Column, TableBuilder},
};

pub fn ui(
    ui: &mut egui::Ui,
    pac: &mut PacState,
    ui_state: &mut SharedUiState,
    tab_state: &mut PkgListState,
) {
    egui::TopBottomPanel::top("top_panel").show_inside(ui, |ui| {
        ui.horizontal(|ui| {
            pac.with_mut(|this| {
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut tab_state.filter_string)
                            .hint_text("🔍 Filter"),
                    )
                    .changed()
                {
                    *this.filt_remote_pkg_list = this
                        .remote_pkg_list
                        .iter()
                        .filter(|pkg| {
                            let filt_lo = tab_state.filter_string.to_ascii_lowercase();
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
                    this.filt_remote_pkg_list.len()
                ));
            });
        });
        ui.add_space(4.0);
    });
    TableBuilder::new(ui)
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::remainder())
        .auto_shrink(false)
        .striped(true)
        .header(18.0, |mut row| {
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
                let list = this.filt_remote_pkg_list;
                body.rows(24.0, list.len(), |mut row| {
                    let pkg = &list[row.index()];
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            let dbname = pkg.db().map_or("<missing db>", |db| db.name());
                            if ui.link(format!("{}/{}", dbname, pkg.name())).clicked() {
                                ui_state
                                    .cmd
                                    .push(Cmd::OpenPkgTab(PkgId::qualified(dbname, pkg.name())));
                            }
                            installed_label_for_remote_pkg(ui, ui_state, pkg, this.local_pkg_list);
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

pub fn installed_label_for_remote_pkg(
    ui: &mut egui::Ui,
    ui_state: &mut SharedUiState,
    remote: &Package,
    local_pkg_list: &[&Package],
) {
    if let Some(local_pkg) = local_pkg_list
        .iter()
        .find(|pkg2| pkg2.name() == remote.name())
    {
        let re = match remote.version().vercmp(local_pkg.version()) {
            std::cmp::Ordering::Less => ui
                .add(
                    egui::Label::new({
                        egui::RichText::new("[older]").color(egui::Color32::ORANGE)
                    })
                    .sense(egui::Sense::click()),
                )
                .on_hover_ui(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("This package is older than the locally installed");
                        ui.label(
                            egui::RichText::new(local_pkg.name()).color(egui::Color32::YELLOW),
                        );
                        ui.label(
                            egui::RichText::new(local_pkg.version().to_string())
                                .color(egui::Color32::ORANGE),
                        );
                    });
                }),
            std::cmp::Ordering::Equal => {
                ui.add(egui::Label::new("[installed]").sense(egui::Sense::click()))
            }
            std::cmp::Ordering::Greater => ui
                .add(
                    egui::Label::new(egui::RichText::new("[newer]").color(egui::Color32::YELLOW))
                        .sense(egui::Sense::click()),
                )
                .on_hover_ui(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("This package is newer than the locally installed");
                        ui.label(
                            egui::RichText::new(local_pkg.name()).color(egui::Color32::YELLOW),
                        );
                        ui.label(
                            egui::RichText::new(local_pkg.version().to_string())
                                .color(egui::Color32::ORANGE),
                        );
                    });
                }),
        };
        if re.hovered() {
            ui.output_mut(|out| out.cursor_icon = egui::CursorIcon::PointingHand);
        }
        if re.clicked() {
            ui_state
                .cmd
                .push(Cmd::OpenPkgTab(PkgId::local(local_pkg.name())));
        }
    }
}

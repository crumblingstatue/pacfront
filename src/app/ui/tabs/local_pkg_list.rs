use {
    super::PkgListState,
    crate::{
        alpm_util::PkgId,
        app::ui::{PacState, SharedUiState, cmd::Cmd},
    },
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
                    *this.filtered_local_pkgs = this
                        .pkg_list
                        .iter()
                        .filter(|pkg| {
                            let filt_lo = tab_state.filter_string.to_ascii_lowercase();
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
                            ui_state.cmd.push(Cmd::OpenPkgTab(PkgId::local(pkg.name())));
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

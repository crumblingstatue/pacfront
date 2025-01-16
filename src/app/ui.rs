use {
    super::PacfrontApp,
    alpm::Package,
    eframe::egui,
    egui_extras::{Column, TableBuilder},
};

pub fn top_panel_ui(app: &mut PacfrontApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.add(egui::TextEdit::singleline(&mut app.ui.filter_string).hint_text("🔍 Filter"))
    });
}

pub fn central_panel_ui(app: &mut PacfrontApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        TableBuilder::new(ui)
            .column(Column::auto())
            .column(Column::remainder())
            .auto_shrink(false)
            .striped(true)
            .header(32.0, |mut row| {
                row.col(|ui| {
                    ui.label("Name");
                });
                row.col(|ui| {
                    ui.label("Description");
                });
            })
            .body(|mut body| {
                body.ui_mut().style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                app.pac.with_pkg_list(|list| {
                    let filtered: Vec<&Package> = list
                        .iter()
                        .filter(|pkg| {
                            pkg.name().contains(&app.ui.filter_string)
                                || pkg
                                    .desc()
                                    .is_some_and(|desc| desc.contains(&app.ui.filter_string))
                        })
                        .copied()
                        .collect();
                    body.rows(24.0, filtered.len(), |mut row| {
                        let pkg = &filtered[row.index()];
                        row.col(|ui| {
                            ui.label(pkg.name());
                        });
                        row.col(|ui| {
                            ui.label(pkg.desc().unwrap_or("<missing description>"));
                        });
                    });
                });
            });
    });
}

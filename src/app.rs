use {
    alpm::{Alpm, Package},
    eframe::egui,
    egui_extras::{Column, TableBuilder},
    ouroboros::self_referencing,
};

pub struct PacfrontApp {
    pac: PacState,
    ui: UiState,
}

#[self_referencing]
struct PacState {
    alpm: Alpm,
    #[borrows(alpm)]
    db: &'this alpm::Db,
    #[borrows(db)]
    #[covariant]
    pkg_list: Vec<&'this Package>,
}

impl PacState {
    fn gimme_new() -> anyhow::Result<Self> {
        let alpm = Alpm::new2("/", "/var/lib/pacman")?;
        let neu = PacStateBuilder {
            alpm,
            db_builder: |this| this.localdb(),
            pkg_list_builder: |db| db.pkgs().into_iter().collect(),
        }
        .build();
        Ok(neu)
    }
}

#[derive(Default)]
struct UiState {
    filter_string: String,
}

impl PacfrontApp {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            pac: PacState::gimme_new()?,
            ui: UiState::default(),
        })
    }
}

impl eframe::App for PacfrontApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add(egui::TextEdit::singleline(&mut self.ui.filter_string).hint_text("🔍 Filter"))
        });
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
                    self.pac.with_pkg_list(|list| {
                        let filtered: Vec<&Package> = list
                            .iter()
                            .filter(|pkg| {
                                pkg.name().contains(&self.ui.filter_string)
                                    || pkg
                                        .desc()
                                        .is_some_and(|desc| desc.contains(&self.ui.filter_string))
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
}

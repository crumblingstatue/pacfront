use {
    super::{PacfrontApp, ouroboros_impl_pac_state::PacState},
    cmd::CmdBuf,
    eframe::egui,
    egui_colors::Colorix,
    egui_dock::{DockArea, DockState},
    std::{
        io::{BufRead, BufReader},
        process::{Command, ExitStatus, Stdio},
        sync::mpsc::TryRecvError,
    },
    tabs::{Tab, TabViewState},
};

pub mod cmd;
mod paint_util;
mod tabs;

pub(super) struct UiState {
    dock_state: DockState<Tab>,
    pub shared: SharedUiState,
}

#[derive(Default)]
pub struct SharedUiState {
    cmd: CmdBuf,
    pub colorix: Option<Colorix>,
    pac_handler: Option<PacChildHandler>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            shared: Default::default(),
            dock_state: DockState::new(Tab::default_tabs()),
        }
    }
}

pub enum PacmanChildEvent {
    Line(std::io::Result<String>),
    Exit(std::io::Result<ExitStatus>),
}

type PacEventRecv = std::sync::mpsc::Receiver<PacmanChildEvent>;

pub struct PacChildHandler {
    recv: Option<PacEventRecv>,
    exit_status: Option<ExitStatus>,
    out_buf: String,
}

impl PacChildHandler {
    pub fn new(recv: PacEventRecv) -> Self {
        Self {
            recv: Some(recv),
            exit_status: None,
            out_buf: String::new(),
        }
    }
}

pub fn top_panel_ui(app: &mut PacfrontApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_panel")
        .exact_height(26.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let (re, painter) =
                    ui.allocate_painter(egui::vec2(24.0, 24.0), egui::Sense::hover());
                paint_util::draw_logo(&painter, re.rect.center(), 8.0);
                ui.label("Pacfront");
                ui.separator();
                ui.menu_button("⟳ Sync", |ui| {
                    if ui.button("🔁 Sync databases (pacman -Sy)").clicked() {
                        ui.close_menu();
                        let mut child = Command::new("pkexec")
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .args(["pacman", "-Sy"])
                            .spawn()
                            .unwrap();
                        let (send, recv) = std::sync::mpsc::channel();
                        app.ui.shared.pac_handler = Some(PacChildHandler::new(recv));
                        let reader = BufReader::new(child.stdout.take().unwrap());
                        let err_reader = BufReader::new(child.stderr.take().unwrap());
                        let send2 = send.clone();
                        std::thread::spawn(move || {
                            for line in reader.lines() {
                                send.send(PacmanChildEvent::Line(line)).unwrap();
                            }
                            send.send(PacmanChildEvent::Exit(child.wait())).unwrap();
                        });
                        std::thread::spawn(move || {
                            for line in err_reader.lines() {
                                send2.send(PacmanChildEvent::Line(line)).unwrap();
                            }
                        });
                    }
                });
                ui.menu_button("☰ Preferences", |ui| {
                    if ui.button("🎨 Color theme").clicked() {
                        ui.close_menu();
                        app.ui.dock_state.push_to_first_leaf(Tab::ColorTheme);
                    }
                    match crate::config::cfg_dir() {
                        Some(dir) => {
                            if ui.button("Open config dir").clicked() {
                                ui.close_menu();
                                let _ = Command::new("xdg-open").arg(dir).status();
                            }
                        }
                        None => {
                            ui.label("<missing config dir>");
                        }
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if app.ui.shared.pac_handler.is_some() {
                        ui.spinner();
                        ui.label("running pacman...");
                    }
                });
            });
        });
    let mut close_handler = false;
    if let Some(handler) = &mut app.ui.shared.pac_handler {
        if let Some(recv) = handler.recv.as_mut() {
            match recv.try_recv() {
                Ok(ev) => match ev {
                    PacmanChildEvent::Line(result) => {
                        handler.out_buf.push_str(&result.unwrap());
                        handler.out_buf.push('\n');
                    }
                    PacmanChildEvent::Exit(exit_status) => {
                        handler.exit_status = Some(exit_status.unwrap())
                    }
                },
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    handler.recv = None;
                }
            }
        }
        if !handler.out_buf.is_empty() {
            egui::Modal::new(egui::Id::new("pacman output modal")).show(ctx, |ui| {
                ui.heading("Pacman output");
                ui.separator();
                let avail_rect = ui.ctx().available_rect();
                let w = (avail_rect.width() * 0.5).round();
                ui.set_width(w);
                egui::ScrollArea::both()
                    .max_height((avail_rect.height() * 0.5).round())
                    .max_width(w)
                    .show(ui, |ui| {
                        ui.set_width(1000.0);
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                        ui.add(
                            egui::TextEdit::multiline(&mut handler.out_buf.as_str())
                                .code_editor()
                                .desired_width(f32::INFINITY),
                        );
                    });
                ui.separator();
                if let Some(status) = &handler.exit_status {
                    ui.label(format!("Pacman exited ({status})"));
                    if ui.button("Close").clicked() {
                        close_handler = true;
                    }
                }
            });
        }
    }
    if close_handler {
        app.ui.shared.pac_handler = None;
    }
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

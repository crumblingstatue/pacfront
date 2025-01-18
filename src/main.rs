#![forbid(unsafe_code)]
#![feature(let_chains, array_windows)]

use {app::PacfrontApp, eframe::NativeOptions};

mod alpm_util;
mod app;
mod config;

fn main() -> anyhow::Result<()> {
    let mut app = PacfrontApp::new()?;
    eframe::run_native(
        "pacfront",
        NativeOptions::default(),
        Box::new(move |cc| {
            app.sync_from_config(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
    .unwrap();
    Ok(())
}

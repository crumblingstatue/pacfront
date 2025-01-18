#![forbid(unsafe_code)]
#![feature(let_chains, array_windows)]

use {app::PacfrontApp, eframe::NativeOptions};

mod alpm_util;
mod app;

fn main() -> anyhow::Result<()> {
    let app = PacfrontApp::new()?;
    eframe::run_native(
        "pacfront",
        NativeOptions::default(),
        Box::new(move |_cc| Ok(Box::new(app))),
    )
    .unwrap();
    Ok(())
}

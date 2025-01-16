#![feature(let_chains)]

use {app::PacfrontApp, eframe::NativeOptions};

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

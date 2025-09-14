#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod highlight;
mod search;
mod io;
mod settings;
mod themes;
mod ui;
mod input;
mod central;
mod style;

use app::FileViewerApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([900.0, 560.0])
            .with_resizable(true)
            .with_title("Gemini File Viewer 2.0.1"),
        ..Default::default()
    };

    eframe::run_native(
        "Gemini File Viewer 2.0.1",
        options,
        Box::new(|cc| Ok(Box::new(FileViewerApp::new(cc))))
    )
}

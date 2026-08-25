//! Steam Achievement Manager, windowed.
//!
//! `windows_subsystem = "windows"` is what stops a console window from appearing
//! behind the app. It is applied only to release builds so that `cargo run`
//! still prints to the terminal while developing.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod args;
mod config;
mod gui;
mod library;
mod search;
mod steam;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let args = args::parse();
    let saved = config::load();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Steam Achievement Manager")
            .with_inner_size([saved.window_width, saved.window_height])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "sam",
        options,
        Box::new(move |cc| Ok(Box::new(gui::SamApp::new(cc, args.id)))),
    )
}

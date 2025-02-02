#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod central_panel;
mod config;
mod config_data;
mod drop_files;
mod image_data;
mod left_side;
mod right_side;
mod top_side;

use crate::app::RoIApp;
use eframe::egui;

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 900.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native("RoI", options, Box::new(|cc| Ok(Box::new(RoIApp::new(cc)))))
}

impl eframe::App for RoIApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_left_side_panel(ctx);
        self.render_right_side_panel(ctx);
        self.render_top_side_panel(ctx);
        self.render_center_panel(ctx);
        self.process_dropped_files(ctx);
    }
}

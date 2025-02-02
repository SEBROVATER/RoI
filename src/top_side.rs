use crate::app::RoIApp;
use std::fs;

impl RoIApp {
    pub fn render_top_side_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top panel").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                if let Some(path) = &self.selected_config {
                    if ui.button("Save current config").clicked() {
                        if let Ok(json_string) =
                            serde_json::to_string_pretty(&self.config_data.config)
                        {
                            if fs::write(path, &json_string).is_ok() {
                                println!("Saved {}", &path.display());
                            } else {
                                eprintln!("Failed saving {}", &path.display());
                            };
                        };
                    };
                }
            });
        });
    }
}

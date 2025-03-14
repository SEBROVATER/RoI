use crate::app::RoIApp;
use crate::config::JsonConfig;
use std::fs::read_to_string;

impl RoIApp {
    pub fn process_dropped_files(&mut self, ctx: &egui::Context) {
        render_files_being_dropped(ctx);

        // Collect dropped files:
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                for dropped_file in &i.raw.dropped_files {
                    if let Some(path) = &dropped_file.path {
                        let ext = path.extension().and_then(|e| e.to_str());
                        match ext {
                            Some("png") | Some("jpg") | Some("jpeg") => {
                                if !self.imgs_paths.contains(path) {
                                    self.imgs_paths.push(path.to_path_buf());
                                }
                            }
                            Some("json") => {
                                if !self.configs_paths.contains(path) {
                                    if let Ok(json_string) = read_to_string(path) {
                                        if serde_json::from_str::<Vec<JsonConfig>>(&json_string)
                                            .is_ok()
                                        {
                                            self.configs_paths.push(path.to_path_buf());
                                        }
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
        })
    }
}

/// Preview hovering files:
pub fn render_files_being_dropped(ctx: &egui::Context) {
    use egui::{Align2, Color32, Id, LayerId, Order, TextStyle};
    use std::fmt::Write as _;

    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let text = ctx.input(|i| {
            let mut text = "Dropping files:\n".to_owned();
            for file in &i.raw.hovered_files {
                if let Some(path) = &file.path {
                    write!(text, "\n{}", path.display()).ok();
                } else if !file.mime.is_empty() {
                    write!(text, "\n{}", file.mime).ok();
                } else {
                    text += "\n???";
                }
            }
            text
        });

        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let screen_rect = ctx.screen_rect();
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            text,
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::WHITE,
        );
    }
}

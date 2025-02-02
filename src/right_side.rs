use crate::app::RoIApp;
use crate::config::JsonConfig;
use crate::config_data::EditCoord;
use egui::TextWrapMode;
use std::fs::read_to_string;

impl RoIApp {
    pub fn render_right_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("configs_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                ui.heading("Configs:");
                if let Some(img_path) = &self.selected_img {
                    if ui.button("create new").clicked() {
                        if let Some(name) = img_path.file_stem() {
                            let str_name = name.to_string_lossy();
                            for i in 1usize..=100usize {
                                let new_name = format!("roi_{}_({}).json", str_name, i);
                                let new_path = img_path.with_file_name(new_name);
                                if new_path.exists() {
                                    continue;
                                } else {
                                    self.configs_paths.push(new_path);
                                    break;
                                };
                            }
                        }
                    }
                };
                let mut to_del: Option<usize> = None;
                for (idx, config_path) in self.configs_paths.iter_mut().enumerate() {
                    let name = config_path.file_name().unwrap().to_string_lossy();
                    let resp = ui
                        .selectable_value(
                            &mut self.selected_config,
                            Some(config_path.to_path_buf()),
                            name,
                        )
                        .on_hover_text(config_path.to_string_lossy());

                    if resp.middle_clicked() {
                        to_del = Some(idx);
                    };
                    if resp.clicked() {
                        if !config_path.exists() {
                            self.config_data = Default::default();
                        } else if let Ok(json_string) = read_to_string(&config_path) {
                            if let Ok(config) =
                                serde_json::from_str::<Vec<JsonConfig>>(&json_string)
                            {
                                self.config_data.config = config;
                                self.config_data.edit_coord = EditCoord::None;
                            }
                        }
                    };
                    if let Some(selected_config) = &self.selected_config {
                        if selected_config == &config_path.to_path_buf() {
                            if let Some(img_data) = &self.img_data {
                                ui.horizontal_top(|ui| {
                                    ui.label("+");
                                    let button = ui.small_button("create new roi");
                                    if button.clicked() {
                                        let [bx1, by1, bx2, by2] = img_data.bounds;
                                        let [x1, y1, x2, y2] = img_data.get_rel_config_coords(
                                            bx1 + 0.3 * (bx2 - bx1),
                                            by1 - 0.3 * (by1 - by2),
                                            bx2 - 0.3 * (bx2 - bx1),
                                            by2 + 0.3 * (by1 - by2),
                                        );
                                        let new_roi = JsonConfig {
                                            x1,
                                            y1,
                                            x2,
                                            y2,
                                            name: String::from("new_roi"),
                                        };
                                        self.config_data.config.push(new_roi);
                                    }
                                });
                            };

                            let mut to_del: Option<usize> = None;
                            for (idx, c) in self.config_data.config.iter_mut().enumerate() {
                                ui.horizontal_top(|ui| {
                                    ui.label(">");
                                    if Some(idx) == self.config_data.edit_idx {
                                        ui.text_edit_singleline(&mut c.name);
                                    } else {
                                        let button = ui.small_button(&c.name);
                                        if button.clicked() {
                                            self.config_data.edit_idx = Some(idx);
                                        };
                                        if button.middle_clicked() {
                                            to_del = Some(idx);
                                        };
                                    };
                                });
                            }
                            if let Some(del_idx) = to_del {
                                self.config_data.safely_remove_roi(del_idx);
                            }
                        }
                    }
                }
                if let Some(idx) = to_del {
                    let removed = self.configs_paths.remove(idx);
                    if Some(removed) == self.selected_config {
                        self.config_data.edit_idx = None;
                        self.config_data.edit_coord = EditCoord::None;
                    };
                }
            });
    }
}

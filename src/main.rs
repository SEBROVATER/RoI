#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

mod config;

use std::any::Any;
use std::fs::{read_to_string, File};
use std::io::{BufReader, Read};
use eframe::egui;
use eframe::epaint::textures::TextureOptions;
use egui::{Color32, ColorImage, Frame, Id, Sense, Stroke, TextureFilter, TextureHandle, Vec2, Vec2b, Window};
use egui_plot::{Line, Plot, PlotImage, PlotItem, PlotPoint, PlotPoints, Polygon};
use kornia::io::functional::read_image_any;
use std::ops::Neg;
use std::path::PathBuf;
use crate::config::JsonConfig;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 800.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "Native file dialogs and drag-and-drop files",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

struct ImageData {
    texture: TextureHandle,
    width: usize,
    height: usize,

}

struct EditConfigData {
    idx: usize,

}

#[derive(Default)]
struct ConfigData {
    config: Vec<JsonConfig>,
    edit_idx: Option<usize>,
}

#[derive(Default)]
struct MyApp {
    imgs_paths: Vec<PathBuf>,
    selected_img: Option<PathBuf>,
    configs_paths: Vec<PathBuf>,
    selected_config: Option<PathBuf>,

    img_data: Option<ImageData>,
    config_data: ConfigData,
    show_relative: bool,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("images_panel").show(ctx, |ui| {
            ui.heading("Images:");
            for img_path in &mut self.imgs_paths {
                let name = img_path.file_name().unwrap().to_string_lossy().to_string();
                if ui
                    .selectable_value(&mut self.selected_img, Some(img_path.to_path_buf()), name)
                    .clicked()
                {
                    if let Ok(img) = read_image_any(img_path) {
                        let color_img =
                            ColorImage::from_rgb([img.width(), img.height()], img.as_slice());

                        let options = TextureOptions {
                            magnification: TextureFilter::Nearest,
                            minification: TextureFilter::Nearest,
                            ..Default::default()
                        };
                        if let Some(img_data) = &mut self.img_data {
                            img_data.texture.set(color_img, options);
                            img_data.width = img.width();
                            img_data.height = img.height();
                        } else {
                            self.img_data = Some(ImageData {
                                texture: ctx.load_texture("current_texture", color_img, options),
                                width: img.width(),
                                height: img.height(),
                            });
                        };
                    }
                };
            }
        });
        egui::SidePanel::right("configs_panel").show(ctx, |ui| {
            ui.heading("Configs:");
            for config_path in &mut self.configs_paths {
                let name = config_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                if ui.selectable_value(
                    &mut self.selected_config,
                    Some(config_path.to_path_buf()),
                    name,
                ).clicked() {
                    if let Ok(json_string) = read_to_string(&config_path) {
                        if let Ok(config) = serde_json::from_str::<Vec<JsonConfig>>(&json_string) {
                            self.config_data.config = config;
                            self.config_data.edit_idx = None;
                        }
                    }
                };
                if let Some(selected_config) = &self.selected_config {
                    if selected_config == &config_path.to_path_buf() {
                        for (idx, c) in self.config_data.config.iter().enumerate() {
                            let is_editable = Some(idx) == self.config_data.edit_idx;
                            ui.horizontal_top(|ui| {
                                ui.label(">");
                                let mut button = ui.small_button(&c.name);
                                if is_editable {
                                    button = button.highlight();
                                }
                                if button.clicked() {
                                    self.config_data.edit_idx = Some(idx);
                                };

                            });
                        }
                    }
                }
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Drag-and-drop files onto the window!");
            let inner_size = ui.available_size();

            if let Some(img_data) = &self.img_data {
                let plot = Plot::new("current_plot")
                    .data_aspect(1.0)
                    .set_margin_fraction(Vec2::new(0., 0.))
                    .height(inner_size.y)
                    .width(inner_size.x)
                    // .y_axis_formatter(|grid, _range| {
                    //     let mut num = grid.value.neg().to_string();
                    //     num.truncate(6);
                    //     num.trim_end_matches('0').to_string()
                    // })
                    .show_grid(Vec2b::new(true, true))
                    .allow_boxed_zoom(false);

                // .auto_bounds(Vec2b::new(false, false))
                let plot_resp = plot.show(ui, |plot_ui| {
                    plot_ui.image(PlotImage::new(
                        img_data.texture.id(),
                        PlotPoint::new(img_data.width as f32 / 2.0, (img_data.height as f32 / 2.0).neg()),
                        Vec2::new(img_data.width as f32, img_data.height as f32),
                    ).allow_hover(false));
                    if let Some(img_data) = &self.img_data {
                        for (idx, config) in self.config_data.config.iter().enumerate() {
                            let mut polygon_obj = Polygon::new(PlotPoints::new(Vec::<[f64; 2]>::from([
                                [config.x1 * img_data.width as f64, (config.y1 * img_data.height as f64).neg()],
                                [config.x2 * img_data.width as f64, (config.y1 * img_data.height as f64).neg()],
                                [config.x2 * img_data.width as f64, (config.y2 * img_data.height as f64).neg()],
                                [config.x1 * img_data.width as f64, (config.y2 * img_data.height as f64).neg()],
                            ]))).highlight(false)
                                .fill_color(Color32::TRANSPARENT)
                                .name(&config.name)
                                .width(1.0)

                                .id(Id::new(idx));


                            plot_ui.polygon(polygon_obj);
                        }
                    }
                });
            };
        });

        preview_files_being_dropped(ctx);

        // Collect dropped files:
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                for dropped_file in &i.raw.dropped_files {
                    // TODO: filter the same paths
                    if let Some(path) = &dropped_file.path {
                        let ext = path.extension().and_then(|e| e.to_str());
                        match ext {
                            Some("png") | Some("jpg") | Some("jpeg") => {
                                self.imgs_paths.push(path.to_path_buf());
                            }
                            Some("json") => {
                                if let Ok(json_string) = read_to_string(path) {
                                    if serde_json::from_str::<Vec<JsonConfig>>(&json_string).is_ok() {
                                        self.configs_paths.push(path.to_path_buf());
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
        });
    }
}

/// Preview hovering files:
fn preview_files_being_dropped(ctx: &egui::Context) {
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

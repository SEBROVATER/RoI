#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

mod config;

use crate::config::JsonConfig;
use eframe::egui;
use eframe::epaint::text::TextWrapMode;
use eframe::epaint::textures::TextureOptions;
use egui::{
    Color32, ColorImage, Id, PointerButton, Stroke, TextBuffer, TextureFilter, TextureHandle, Vec2,
    Vec2b,
};
use egui_plot::{
    Axis, AxisHints, HLine, HPlacement, Plot, PlotImage, PlotItem, PlotPoint, PlotPoints, Polygon,
    VLine, VPlacement,
};
use kornia::io::functional::read_image_any;
use std::cmp::Ordering;
use std::fs::read_to_string;
use std::ops::Neg;
use std::path::PathBuf;
use std::{cmp, fs};

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
    bounds: [f64; 4],
}
impl ImageData {
    pub fn get_rel_config_coords(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> [f64; 4] {
        [
            self.get_rel_config_coord_x1(f64::min(x1, x2)),
            self.get_rel_config_coord_y1(f64::max(y1, y2)),
            self.get_rel_config_coord_x2(f64::max(x1, x2)),
            self.get_rel_config_coord_y2(f64::min(y1, y2)),
        ]
    }
    pub fn get_rel_config_coord_x1(&self, x1: f64) -> f64 {
        (x1.floor() / self.width as f64).clamp(0.0, 1.0)
    }
    pub fn get_rel_config_coord_y1(&self, y1: f64) -> f64 {
        (y1.neg().ceil() / self.height as f64).clamp(0.0, 1.0)
    }
    pub fn get_rel_config_coord_x2(&self, x2: f64) -> f64 {
        (x2.ceil() / self.width as f64).clamp(0.0, 1.0)
    }
    pub fn get_rel_config_coord_y2(&self, y2: f64) -> f64 {
        (y2.neg().floor() / self.height as f64).clamp(0.0, 1.0)
    }
}

struct EditConfigData {
    idx: usize,
}

enum EditCoord {
    X1,
    Y1,
    X2,
    Y2,
    None,
}
impl Default for EditCoord {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default)]
struct ConfigData {
    config: Vec<JsonConfig>,
    edit_idx: Option<usize>,
    edit_coord: EditCoord,
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
            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
            ui.heading("Images:");
            let mut to_del: Option<usize> = None;
            for (idx, img_path) in self.imgs_paths.iter_mut().enumerate() {
                let name = img_path.file_name().unwrap().to_string_lossy();
                let mut resp = ui
                    .selectable_value(&mut self.selected_img, Some(img_path.to_path_buf()), name)
                    .on_hover_text(img_path.to_string_lossy());
                if resp.middle_clicked() {
                    to_del = Some(idx);
                };
                if resp.clicked() {
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
                                bounds: [0.0, 0.0, img.width() as f64, img.height() as f64],
                            });
                        };
                    }
                };
            }
            if let Some(idx) = to_del {
                let removed = self.imgs_paths.remove(idx);
                if Some(removed) == self.selected_img {
                    self.selected_img = None;
                    self.img_data = None;
                    self.config_data.edit_idx = None;
                    self.config_data.edit_coord = EditCoord::None;
                };
            }
        });
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
                    let mut resp = ui
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
                                self.config_data.config.remove(del_idx);
                                if let Some(edit_idx) = self.config_data.edit_idx {
                                    match del_idx.cmp(&edit_idx) {
                                        Ordering::Less => {
                                            self.config_data.edit_idx = Some(edit_idx - 1);
                                            self.config_data.edit_coord = EditCoord::None;
                                        }
                                        Ordering::Equal => {
                                            self.config_data.edit_idx = None;
                                            self.config_data.edit_coord = EditCoord::None;
                                        }
                                        Ordering::Greater => {}
                                    }
                                }
                            };
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
        egui::TopBottomPanel::top("top panel").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                if let Some(path) = &self.selected_config {
                    if ui.button("Save current config").clicked() {
                        if let Ok(json_string) = serde_json::to_string(&self.config_data.config) {
                            if let Ok(_) = fs::write(path, &json_string) {
                                println!("Saved {}", &path.display());
                            } else {
                                eprintln!("Failed saving {}", &path.display());
                            };
                        };
                    };
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            // TODO: show help until some image is dropped
            let inner_size = ui.available_size();

            if let Some(img_data) = &mut self.img_data {
                let plot = Plot::new("current_plot")
                    .data_aspect(1.0)
                    .set_margin_fraction(Vec2::new(0., 0.))
                    .height(inner_size.y)
                    .width(inner_size.x)
                    .label_formatter(|name, value| {
                        if !(0.0..=img_data.width as f64).contains(&value.x)
                            || !((img_data.height as f64).neg()..=0.0).contains(&value.y)
                        {
                            return "".to_string();
                        }
                        let xi = value.x;
                        let mut xf = (xi / img_data.width as f64).to_string();
                        xf.truncate(6);
                        let yi = value.y.neg();
                        let mut yf = (yi / img_data.height as f64).to_string();
                        yf.truncate(6);
                        format!(
                            "x = {:.1} | {}\ny = {:.1} | {}\n",
                            xi,
                            xf.trim_end_matches('0'),
                            yi,
                            yf.trim_end_matches('0')
                        )
                    })
                    .show_grid(Vec2b::new(true, true))
                    .allow_boxed_zoom(false)
                    .x_axis_position(VPlacement::Top)
                    .custom_x_axes(vec![
                        AxisHints::new_x().placement(VPlacement::Top),
                        AxisHints::new_x().formatter(|grid, _range| {
                            let mut val = (grid.value / img_data.width as f64).to_string();
                            val.truncate(6);
                            val.trim_end_matches('0').to_string()
                        }),
                    ])
                    .custom_y_axes(vec![
                        AxisHints::new_y().placement(HPlacement::Right).formatter(
                            |grid, _range| {
                                let mut val =
                                    (grid.value.neg() / img_data.height as f64).to_string();
                                val.truncate(6);
                                val.trim_end_matches('0').to_string()
                            },
                        ),
                        AxisHints::new_y().formatter(|grid, _range| grid.value.neg().to_string()),
                    ]);

                let plot_resp = plot.show(ui, |plot_ui| {
                    let plot_img = PlotImage::new(
                        img_data.texture.id(),
                        PlotPoint::new(
                            img_data.width as f32 / 2.0,
                            (img_data.height as f32 / 2.0).neg(),
                        ),
                        Vec2::new(img_data.width as f32, img_data.height as f32),
                    )
                    .allow_hover(false);
                    plot_ui.image(plot_img);

                    for (idx, config) in self.config_data.config.iter().enumerate() {
                        let [x1, y1, x2, y2] = config
                            .get_abs_plot_coords(img_data.width as f64, img_data.height as f64);

                        if &Some(idx) == &self.config_data.edit_idx {
                            plot_ui.vline(
                                VLine::new(x1)
                                    .highlight(matches!(self.config_data.edit_coord, EditCoord::X1))
                                    .stroke(Stroke::new(2.0, Color32::GREEN)),
                            );
                            plot_ui.hline(
                                HLine::new(y1)
                                    .highlight(matches!(self.config_data.edit_coord, EditCoord::Y1))
                                    .stroke(Stroke::new(2.0, Color32::GREEN)),
                            );
                            plot_ui.vline(
                                VLine::new(x2)
                                    .highlight(matches!(self.config_data.edit_coord, EditCoord::X2))
                                    .stroke(Stroke::new(2.0, Color32::GREEN)),
                            );
                            plot_ui.hline(
                                HLine::new(y2)
                                    .highlight(matches!(self.config_data.edit_coord, EditCoord::Y2))
                                    .stroke(Stroke::new(2.0, Color32::GREEN)),
                            );
                        } else {
                            let polygon_obj =
                                Polygon::new(PlotPoints::new(Vec::<[f64; 2]>::from([
                                    [x1, y1],
                                    [x2, y1],
                                    [x2, y2],
                                    [x1, y2],
                                ])))
                                .fill_color(Color32::TRANSPARENT)
                                .name(&config.name)
                                .stroke(Stroke::new(2.0, Color32::WHITE))
                                .id(Id::new(idx));

                            plot_ui.polygon(polygon_obj);
                        }
                    }
                });

                let bounds = plot_resp.transform.bounds();
                let min: [f64; 2] = bounds.min();
                let max: [f64; 2] = bounds.max();
                // account inverted y axis
                img_data.bounds = [min[0], max[1], max[0], min[1]];

                if plot_resp.response.middle_clicked() {
                    if let Some(pos) = ctx.pointer_interact_pos() {
                        let plot_pos = plot_resp.transform.value_from_position(pos);
                        let x = img_data.get_rel_config_coord_x1(plot_pos.x);
                        let y = img_data.get_rel_config_coord_y1(plot_pos.y);

                        let mut best_match_idx: Option<usize> = None;
                        let mut best_center_dist = f64::MAX;

                        for (idx, config) in self.config_data.config.iter().enumerate() {
                            if (config.x1..=config.x2).contains(&x)
                                && (config.y1..config.y2).contains(&y)
                            {
                                let (cx, cy) =
                                    ((config.x2 + config.x1) / 2.0, (config.y2 + config.y1) / 2.0);
                                let dist = ((cx - x).powi(2) + (cy - y).powi(2)).sqrt();
                                if dist < best_center_dist {
                                    best_center_dist = dist;
                                    best_match_idx = Some(idx);
                                }
                            }
                        }

                        if let Some(del_idx) = best_match_idx {
                            self.config_data.config.remove(del_idx);
                            if let Some(edit_idx) = self.config_data.edit_idx {
                                match del_idx.cmp(&edit_idx) {
                                    Ordering::Less => {
                                        self.config_data.edit_idx = Some(edit_idx - 1);
                                        self.config_data.edit_coord = EditCoord::None;
                                    }
                                    Ordering::Equal => {
                                        self.config_data.edit_idx = None;
                                        self.config_data.edit_coord = EditCoord::None;
                                    }
                                    Ordering::Greater => {}
                                }
                            }
                        }
                    }
                };

                if plot_resp.response.secondary_clicked() {
                    if let Some(pos) = ctx.pointer_interact_pos() {
                        let plot_pos = plot_resp.transform.value_from_position(pos);
                        let x = img_data.get_rel_config_coord_x1(plot_pos.x);
                        let y = img_data.get_rel_config_coord_y1(plot_pos.y);

                        let mut best_match_idx: Option<usize> = None;
                        let mut best_center_dist = f64::MAX;

                        for (idx, config) in self.config_data.config.iter().enumerate() {
                            if (config.x1..=config.x2).contains(&x)
                                && (config.y1..config.y2).contains(&y)
                            {
                                let (cx, cy) =
                                    ((config.x2 + config.x1) / 2.0, (config.y2 + config.y1) / 2.0);
                                let dist = ((cx - x).powi(2) + (cy - y).powi(2)).sqrt();
                                if dist < best_center_dist {
                                    best_center_dist = dist;
                                    best_match_idx = Some(idx);
                                }
                            }
                        }
                        if best_match_idx.is_some() {
                            self.config_data.edit_idx = best_match_idx;
                        }
                    }
                };
                if plot_resp.response.drag_started_by(PointerButton::Secondary) {
                    if let Some(pos) = ctx.pointer_interact_pos() {
                        let plot_pos = plot_resp.transform.value_from_position(pos);

                        if let Some(idx) = self.config_data.edit_idx {
                            let config = &self.config_data.config[idx];
                            let [x1, y1, x2, y2] = config
                                .get_abs_plot_coords(img_data.width as f64, img_data.height as f64);

                            let mut best_match = EditCoord::None;
                            let mut best_val = f64::MAX;

                            let mut val: f64 = (x1 - plot_pos.x).abs();
                            if (val < 10.0) && (val < best_val) {
                                best_match = EditCoord::X1;
                                best_val = val;
                            }
                            val = (y1 - plot_pos.y).abs();
                            if (val < 10.0) && (val < best_val) {
                                best_match = EditCoord::Y1;
                                best_val = val;
                            }
                            val = (x2 - plot_pos.x).abs();
                            if (val < 10.0) && (val < best_val) {
                                best_match = EditCoord::X2;
                                best_val = val;
                            }
                            val = (y2 - plot_pos.y).abs();
                            if (val < 10.0) && (val < best_val) {
                                best_match = EditCoord::Y2;
                                best_val = val;
                            }
                            self.config_data.edit_coord = best_match;
                            // TODO: highlight editing coord
                        }
                    }
                }
                if plot_resp.response.drag_stopped_by(PointerButton::Secondary) {
                    self.config_data.edit_coord = EditCoord::None;
                }
                if plot_resp.response.dragged_by(PointerButton::Secondary) {
                    if let Some(pos) = ctx.pointer_interact_pos() {
                        let plot_pos = plot_resp.transform.value_from_position(pos);

                        if let Some(idx) = self.config_data.edit_idx {
                            let config = &mut self.config_data.config[idx];
                            match self.config_data.edit_coord {
                                EditCoord::X1 => {
                                    config.x1 =
                                        img_data.get_rel_config_coord_x1(plot_pos.x).min(config.x2)
                                }
                                EditCoord::Y1 => {
                                    config.y1 =
                                        img_data.get_rel_config_coord_y1(plot_pos.y).min(config.y2)
                                }
                                EditCoord::X2 => {
                                    config.x2 =
                                        img_data.get_rel_config_coord_x2(plot_pos.x).max(config.x1)
                                }
                                EditCoord::Y2 => {
                                    config.y2 =
                                        img_data.get_rel_config_coord_y2(plot_pos.y).max(config.y1)
                                }
                                EditCoord::None => {}
                            }
                        }
                    }
                }
            } else {
                ui.label("Drag-and-drop files onto the window!");
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
                                if !self.imgs_paths.contains(&path) {
                                    self.imgs_paths.push(path.to_path_buf());
                                }
                            }
                            Some("json") => {
                                if !self.configs_paths.contains(&path) {
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

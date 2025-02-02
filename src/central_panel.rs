use crate::app::RoIApp;
use crate::config_data::EditCoord;
use egui::{Color32, Id, PointerButton, RichText, Stroke, Vec2, Vec2b};
use egui_plot::{
    AxisHints, HLine, HPlacement, Plot, PlotImage, PlotPoint, PlotPoints, Polygon, VLine,
    VPlacement,
};
use std::ops::Neg;

impl RoIApp {
    pub fn render_center_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let inner_size = ui.available_size();

            if let Some(img_data) = &mut self.img_data {
                let plot = Plot::new("current_plot")
                    .data_aspect(1.0)
                    .set_margin_fraction(Vec2::new(0., 0.))
                    .height(inner_size.y)
                    .width(inner_size.x)
                    .label_formatter(|_name, value| {
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

                        if Some(idx) == self.config_data.edit_idx {
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
                // account inverted y-axis
                img_data.bounds = [min[0], max[1], max[0], min[1]];

                if plot_resp.response.middle_clicked() {
                    if let Some(pos) = ctx.pointer_interact_pos() {
                        let plot_pos = plot_resp.transform.value_from_position(pos);
                        let x = img_data.get_rel_config_coord_x1(plot_pos.x);
                        let y = img_data.get_rel_config_coord_y1(plot_pos.y);

                        if let Some(del_idx) = self.config_data.find_relevant_roi_at_coord(x, y) {
                            self.config_data.safely_remove_roi(del_idx);
                        }
                    }
                };

                if plot_resp.response.secondary_clicked() {
                    if let Some(pos) = ctx.pointer_interact_pos() {
                        let plot_pos = plot_resp.transform.value_from_position(pos);
                        let x = img_data.get_rel_config_coord_x1(plot_pos.x);
                        let y = img_data.get_rel_config_coord_y1(plot_pos.y);

                        let best_match_idx = self.config_data.find_relevant_roi_at_coord(x, y);
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
                                // best_val = val;
                            }
                            self.config_data.edit_coord = best_match;
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
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new(
                            "To add files, drag-and-drop them onto the window.

                            RightClick - to select and edit bboxes in the image.

                            MiddleClick - to remove any elements: images, configs, bboxes.

                            Click on editable config to reset changes.",
                        )
                        .heading()
                        .size(25.0),
                    );
                });
            };
        });
    }
}

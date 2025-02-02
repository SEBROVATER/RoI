use crate::app::RoIApp;
use crate::config_data::EditCoord;
use crate::image_data::ImageData;
use egui::{ColorImage, TextWrapMode, TextureFilter, TextureOptions};
use kornia::io::functional::read_image_any;

impl RoIApp {
    pub fn render_left_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("images_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                ui.heading("Images:");
                let mut to_del: Option<usize> = None;
                for (idx, img_path) in self.imgs_paths.iter_mut().enumerate() {
                    let name = img_path.file_name().unwrap().to_string_lossy();
                    let resp = ui
                        .selectable_value(
                            &mut self.selected_img,
                            Some(img_path.to_path_buf()),
                            name,
                        )
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
                                    texture: ctx.load_texture(
                                        "current_texture",
                                        color_img,
                                        options,
                                    ),
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
    }
}

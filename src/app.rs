use crate::config_data::ConfigData;
use crate::image_data::ImageData;
use std::path::PathBuf;

#[derive(Default)]
pub struct RoIApp {
    pub imgs_paths: Vec<PathBuf>,
    pub selected_img: Option<PathBuf>,
    pub configs_paths: Vec<PathBuf>,
    pub selected_config: Option<PathBuf>,

    pub img_data: Option<ImageData>,
    pub config_data: ConfigData,
}
impl RoIApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_pixels_per_point(1.2);

        Default::default()
    }
}

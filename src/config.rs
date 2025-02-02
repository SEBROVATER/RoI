use serde::{Deserialize, Serialize};
use std::ops::Neg;

#[derive(Serialize, Deserialize)]
pub struct JsonConfig {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub name: String,
}

impl JsonConfig {
    pub fn get_center(&self) -> [f64; 2] {
        [(self.x2 + self.x1) / 2.0, (self.y2 + self.y1) / 2.0]
    }
    pub fn get_abs_plot_coords(&self, img_width: f64, img_height: f64) -> [f64; 4] {
        let x1 = (self.x1 * img_width).floor();
        let y1 = (self.y1 * img_height).neg().ceil();
        let x2 = (self.x2 * img_width).ceil();
        let y2 = (self.y2 * img_height).neg().floor();
        [x1, y1, x2, y2]
    }
}

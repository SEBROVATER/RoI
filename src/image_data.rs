use eframe::epaint::TextureHandle;
use std::ops::Neg;

pub struct ImageData {
    pub texture: TextureHandle,
    pub width: usize,
    pub height: usize,
    pub bounds: [f64; 4],
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

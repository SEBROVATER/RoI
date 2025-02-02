use crate::config::JsonConfig;
use std::cmp::Ordering;

#[derive(Default)]
pub struct ConfigData {
    pub config: Vec<JsonConfig>,
    pub edit_idx: Option<usize>,
    pub edit_coord: EditCoord,
}
impl ConfigData {
    pub fn find_relevant_roi_at_coord(&self, x: f64, y: f64) -> Option<usize> {
        let mut best_match_idx: Option<usize> = None;
        let mut best_center_dist = f64::MAX;

        for (idx, config) in self.config.iter().enumerate() {
            if (config.x1..=config.x2).contains(&x) && (config.y1..config.y2).contains(&y) {
                let (cx, cy) = ((config.x2 + config.x1) / 2.0, (config.y2 + config.y1) / 2.0);
                let dist = ((cx - x).powi(2) + (cy - y).powi(2)).sqrt();
                if dist < best_center_dist {
                    best_center_dist = dist;
                    best_match_idx = Some(idx);
                }
            }
        }
        best_match_idx
    }
    pub fn safely_remove_roi(&mut self, idx: usize) {
        if idx >= self.config.len() {
            return;
        }
        self.config.remove(idx);
        if let Some(edit_idx) = self.edit_idx {
            match idx.cmp(&edit_idx) {
                Ordering::Less => {
                    self.edit_idx = Some(edit_idx - 1);
                    self.edit_coord = EditCoord::None;
                }
                Ordering::Equal => {
                    self.edit_idx = None;
                    self.edit_coord = EditCoord::None;
                }
                Ordering::Greater => {}
            }
        }
    }
}
pub enum EditCoord {
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

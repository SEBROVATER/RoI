use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct JsonConfig {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub name: String,
}
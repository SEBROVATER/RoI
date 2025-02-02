use crate::config::JsonConfig;

#[derive(Default)]
pub struct ConfigData {
    pub config: Vec<JsonConfig>,
    pub edit_idx: Option<usize>,
    pub edit_coord: EditCoord,
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

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Alignment {
    Left,
    Center,
    Right,
    Justified,
}

impl Default for Alignment {
    fn default() -> Self {
        Alignment::Left
    }
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Margin {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
}

impl Default for Margin {
    fn default() -> Self {
        Margin {
            top: 0,
            bottom: 0,
            left: 0,
            right: 0,
        }
    }
}

impl Margin {
    pub fn new(top: u32, bottom: u32, left: u32, right: u32) -> Margin {
        Margin {
            top,
            bottom,
            left,
            right,
        }
    }
}

impl Margin {
    pub fn with_top(self, top: u32) -> Margin {
        Margin { top, ..self }
    }

    pub fn with_bottom(self, bottom: u32) -> Margin {
        Margin { bottom, ..self }
    }

    pub fn with_left(self, left: u32) -> Margin {
        Margin { left, ..self }
    }

    pub fn with_right(self, right: u32) -> Margin {
        Margin { right, ..self }
    }
}

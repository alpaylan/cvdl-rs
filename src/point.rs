use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Point { x, y }
    }

    pub fn move_x_by(&self, x: i32) -> Self {
        Point {
            x: self.x.checked_add_signed(x).unwrap(),
            y: self.y,
        }
    }

    pub fn move_y_by(&self, y: i32) -> Self {
        Point {
            x: self.x,
            y: self.y.checked_add_signed(y).unwrap(),
        }
    }

    pub fn move_x_to(&self, x: u32) -> Self {
        Point { x, y: self.y }
    }

    pub fn move_y_to(&self, y: u32) -> Self {
        Point { x: self.x, y }
    }
}

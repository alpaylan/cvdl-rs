use crate::point::Point;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpatialBox {
    pub top_left: Point,
    pub bottom_right: Point,
}

impl SpatialBox {
    pub fn new(top_left: Point, bottom_right: Point) -> Self {
        SpatialBox {
            top_left,
            bottom_right,
        }
    }

    pub fn move_x_by(&self, x: i32) -> Self {
        SpatialBox {
            top_left: self.top_left.move_x_by(x),
            bottom_right: self.bottom_right.move_x_by(x),
        }
    }

    pub fn move_y_by(&self, y: i32) -> Self {
        SpatialBox {
            top_left: self.top_left.move_y_by(y),
            bottom_right: self.bottom_right.move_y_by(y),
        }
    }

    pub fn width(&self) -> u32 {
        self.bottom_right.x - self.top_left.x + 1
    }

    pub fn height(&self) -> u32 {
        self.bottom_right.y - self.top_left.y + 1
    }

    pub fn frame_points(&self) -> Vec<Point> {
        let mut points = Vec::new();

        for x in self.top_left.x..=self.bottom_right.x {
            points.push(Point {
                x,
                y: self.top_left.y,
            });
            points.push(Point {
                x,
                y: self.bottom_right.y,
            });
        }

        for y in self.top_left.y..=self.bottom_right.y {
            points.push(Point {
                x: self.top_left.x,
                y,
            });
            points.push(Point {
                x: self.bottom_right.x,
                y,
            });
        }

        points
    }

    pub fn inner_points(&self) -> Vec<Point> {
        let mut points = Vec::new();
        for x in self.top_left.x + 1..self.bottom_right.x {
            for y in self.top_left.y + 1..self.bottom_right.y {
                points.push(Point { x, y });
            }
        }

        points
    }

    pub fn inner_box(&self) -> SpatialBox {
        SpatialBox {
            top_left: Point {
                x: self.top_left.x + 1,
                y: self.top_left.y + 1,
            },
            bottom_right: Point {
                x: self.bottom_right.x - 1,
                y: self.bottom_right.y - 1,
            },
        }
    }
}

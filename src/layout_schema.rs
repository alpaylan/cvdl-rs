use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LayoutSchema {
    #[serde(rename = "schema-name")]
    pub schema_name: String,
    #[serde(rename = "header-layout-schema")]
    pub header_layout_schema: Layout,
    #[serde(rename = "item-layout-schema")]
    pub item_layout_schema: Layout,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Layout {
    Stack(Vec<Layout>),
    Grid(LayoutGrid),
    Element(Element),
}

impl Layout {
    pub fn diagnostic(&self) -> String {
        match self {
            Layout::Stack(_) => String::from("Stack"),
            Layout::Grid(_) => String::from("Grid"),
            Layout::Element(_) => String::from("Element"),
        }
    }
}

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

impl Layout {
    fn compute_height(&self) -> u32 {
        match self {
            Layout::Stack(stacks) => stacks.iter().map(|e| e.compute_height()).sum::<u32>() + 2,
            Layout::Grid(grid) => {
                grid.elements
                    .iter()
                    .map(|e| e.margin.top + e.margin.bottom + e.element.compute_height())
                    .max()
                    .unwrap_or_default()
                    + 2
            }
            Layout::Element(_) => 1 + 2,
        }
    }

    fn compute_total_elements_width(&self) -> u32 {
        match self {
            Layout::Stack(stacks) => {
                stacks
                    .iter()
                    .map(|e| e.compute_total_elements_width())
                    .max()
                    .unwrap()
                    + 2
            }
            Layout::Grid(grid) => grid
                .elements
                .iter()
                .map(|e| e.margin.left + e.margin.right + e.element.compute_total_elements_width())
                .sum::<u32>() + 2,
            Layout::Element(elem) => elem.text.len() as u32 + 2,
        }
    }

    pub fn blueprint(&self) -> String {
        let mut blueprintmap: HashMap<Point, String> = HashMap::new();
        let height = self.compute_height();
        let width = 100;

        self.compute_blueprint(
            &mut blueprintmap,
            SpatialBox::new(Point::new(0, 0), Point::new(width - 1, height - 1)),
        );
        let mut blueprintstring = String::new();

        blueprintstring.push(' ');
        blueprintstring.push(' ');
        for j in 0..width {
            blueprintstring.push(j.to_string().chars().last().unwrap());
        }
        blueprintstring.push('\n');
        for i in 0..height {
            blueprintstring.push(i.to_string().chars().last().unwrap());
            blueprintstring.push(' ');
            for j in 0..width {
                if let Some(c) = blueprintmap.get(&Point::new(j, i)) {
                    blueprintstring.push(c.chars().next().unwrap());
                } else {
                    blueprintstring.push(' ')
                }
            }
            blueprintstring.push('\n');
        }

        blueprintstring
    }

    fn draw_outer_box(
        &self,
        blueprintmap: &mut HashMap<Point, String>,
        current_box: SpatialBox,
    ) -> SpatialBox {
        
        let marker = match self {
            Layout::Stack(_) => "+",
            Layout::Grid(_) => "*",
            Layout::Element(_) => "-",
        };

        for point in current_box.frame_points() {
            blueprintmap.insert(point, marker.to_string());
        }

        current_box.inner_box()
    }

    fn compute_blueprint(
        &self,
        mut blueprintmap: &mut HashMap<Point, String>,
        current_box: SpatialBox,
    ) {
        let mut current_box = self.draw_outer_box(&mut blueprintmap, current_box);

        match self {
            Layout::Stack(layouts) => {
                for layout in layouts {
                    let height = layout.compute_height();
                    let new_box = SpatialBox::new(
                        current_box.top_left,
                        Point::new(
                            current_box.bottom_right.x,
                            current_box.top_left.y + height - 1,
                        ),
                    );
                    layout.compute_blueprint(&mut blueprintmap, new_box);
                    current_box.top_left.y += height;
                }
            }
            Layout::Grid(grid) => {
                let elements_width: u32 = self.compute_total_elements_width() - 2;

                match grid.alignment {
                    Alignment::Left => {
                        // todo: Bounds check
                        // Explanation: When you too large margins, elements
                        // go out of the picture. There should be a bounce check.
                        for element in &grid.elements {
                            let height = element.element.compute_height();
                            let width = element.element.compute_total_elements_width();

                            let new_box = SpatialBox::new(
                                current_box.top_left,
                                current_box
                                    .top_left
                                    .move_x_by((width - 1) as i32)
                                    .move_y_by((height - 1) as i32),
                            )
                            .move_x_by(element.margin.left as i32)
                            .move_y_by(element.margin.top as i32);

                            element
                                .element
                                .compute_blueprint(&mut blueprintmap, new_box);
                            current_box.top_left = current_box.top_left.move_x_by(
                                (width + element.margin.right + element.margin.left) as i32,
                            );
                        }
                    }
                    Alignment::Center => {
                        let total_width = current_box.width();
                        let left_spacing: u32 = (total_width - elements_width) / 2;
                        let mut current_box = SpatialBox::new(
                            current_box.top_left.move_x_by(left_spacing as i32),
                            current_box.bottom_right,
                        );
                        for element in &grid.elements {
                            let height = element.element.compute_height();
                            let width = element.element.compute_total_elements_width();
                            let new_box = SpatialBox::new(
                                current_box.top_left,
                                current_box
                                    .top_left
                                    .move_x_by((width - 1) as i32)
                                    .move_y_by((height - 1) as i32),
                            )
                            .move_x_by(element.margin.left as i32)
                            .move_y_by(element.margin.top as i32);
                            element
                                .element
                                .compute_blueprint(&mut blueprintmap, new_box);
                            current_box.top_left = current_box.top_left.move_x_by(
                                (width + element.margin.right + element.margin.left) as i32,
                            )
                        }
                    }
                    Alignment::Right => {
                        let left_spacing: u32 = current_box.width() - elements_width;

                        let mut current_box = SpatialBox::new(
                            current_box.top_left.move_x_by(left_spacing as i32),
                            current_box.bottom_right,
                        );
                        for element in &grid.elements {
                            let height = element.element.compute_height();
                            let width = element.element.compute_total_elements_width();
                            let new_box = SpatialBox::new(
                                current_box.top_left,
                                current_box
                                    .top_left
                                    .move_x_by((width - 1) as i32)
                                    .move_y_by((height - 1) as i32),
                            )
                            .move_x_by(element.margin.left as i32)
                            .move_y_by(element.margin.top as i32);
                            element
                                .element
                                .compute_blueprint(&mut blueprintmap, new_box);
                            current_box.top_left = current_box.top_left.move_x_by(
                                (width + element.margin.right + element.margin.left) as i32,
                            );
                        }
                    }
                    Alignment::Justified => {
                        let total_spacing: u32 = current_box.width() - elements_width;
                        let per_element_spacing = total_spacing / (grid.elements.len() - 1) as u32;
                        for element in &grid.elements {
                            let height = element.element.compute_height();
                            let width = element.element.compute_total_elements_width();
                            let new_box = SpatialBox::new(
                                current_box.top_left,
                                current_box
                                    .top_left
                                    .move_x_by((width - 1) as i32)
                                    .move_y_by((height - 1) as i32),
                            )
                            .move_x_by(element.margin.left as i32)
                            .move_y_by(element.margin.top as i32);

                            element
                                .element
                                .compute_blueprint(&mut blueprintmap, new_box);
                            current_box.top_left = current_box.top_left.move_x_by(
                                (width
                                    + per_element_spacing
                                    + element.margin.right
                                    + element.margin.left) as i32,
                            );
                        }
                    }
                }
            }
            Layout::Element(elem) => {
                for (i, c) in elem.text.chars().enumerate() {
                    blueprintmap.insert(current_box.top_left.move_x_by(i as i32), c.to_string());
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LayoutGrid {
    pub elements: Vec<GridElement>,
    pub alignment: Alignment,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GridElement {
    pub min_width: Option<u32>,
    pub max_width: Option<u32>,
    pub default_width: Option<u32>,
    pub margin: Margin,
    pub element: Layout,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Element {
    pub text: String,
    pub font: Font,
    pub alignment: Alignment,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Font {
    pub name: String,
    pub size: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_blueprint() {

    let layout = Layout::Stack(vec![
        Layout::Grid(
            LayoutGrid { elements: vec![
                GridElement { min_width: None, max_width: None, default_width: None, margin: Margin { top: 3, bottom: 0, left: 20, right: 0 }, element: Layout::Element(Element { text: String::from("A"), font: Font { name: String::from("Arial"), size: 12 }, alignment: Alignment::Left }) }
                , GridElement { min_width: None, max_width: None, default_width: None, margin: Margin { top: 0, bottom: 2, left: 0, right: 20 }, element: Layout::Element(Element { text: String::from("B"), font: Font { name: String::from("Arial"), size: 12 }, alignment: Alignment::Left }) }
            ]
                , alignment: Alignment::Justified }
        ),
        Layout::Grid(
            LayoutGrid { elements: vec![
                GridElement { min_width: None, max_width: None, default_width: None, margin: Margin { top: 3, bottom: 0, left: 20, right: 0 }, element: Layout::Element(Element { text: String::from("A"), font: Font { name: String::from("Arial"), size: 12 }, alignment: Alignment::Left }) }
                , GridElement { min_width: None, max_width: None, default_width: None, margin: Margin { top: 0, bottom: 2, left: 0, right: 20 }, element: Layout::Element(Element { text: String::from("B"), font: Font { name: String::from("Arial"), size: 12 }, alignment: Alignment::Left }) }
            ]
                , alignment: Alignment::Left }
        ),
        Layout::Grid(
            LayoutGrid { elements: vec![
                GridElement { min_width: None, max_width: None, default_width: None, margin: Margin { top: 3, bottom: 0, left: 20, right: 0 }, element: Layout::Element(Element { text: String::from("A"), font: Font { name: String::from("Arial"), size: 12 }, alignment: Alignment::Left }) }
                , GridElement { min_width: None, max_width: None, default_width: None, margin: Margin { top: 0, bottom: 2, left: 0, right: 20 }, element: Layout::Element(Element { text: String::from("B"), font: Font { name: String::from("Arial"), size: 12 }, alignment: Alignment::Left }) }
            ]
                , alignment: Alignment::Right }
        ),
        Layout::Grid(
            LayoutGrid { elements: vec![
                GridElement { min_width: None, max_width: None, default_width: None, margin: Margin { top: 3, bottom: 0, left: 20, right: 0 }, element: Layout::Element(Element { text: String::from("A"), font: Font { name: String::from("Arial"), size: 12 }, alignment: Alignment::Left }) }
                , GridElement { min_width: None, max_width: None, default_width: None, margin: Margin { top: 0, bottom: 2, left: 0, right: 20 }, element: Layout::Element(Element { text: String::from("B"), font: Font { name: String::from("Arial"), size: 12 }, alignment: Alignment::Left }) }
            ]
                , alignment: Alignment::Center }
        )
    ]);
    }
}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use std::{io::ErrorKind, path::Path};

use std::fs;
use std::io::Error;

use crate::data_schema::{DataSchema, Field};
use crate::point::Point;
use crate::resume_data::{ItemContent, ResumeData};
use crate::spatial_box::SpatialBox;

#[derive(Serialize, Deserialize, Debug)]
pub struct LayoutSchema {
    #[serde(rename = "schema-name")]
    pub schema_name: String,
    #[serde(rename = "header-layout-schema")]
    pub header_layout_schema: Layout,
    #[serde(rename = "item-layout-schema")]
    pub item_layout_schema: Layout,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

    pub fn just_grid(grid_elements: Vec<Layout>) -> Layout {
        Layout::Grid(LayoutGrid {
            elements: grid_elements
                .iter()
                .map(|elem| GridElement::default(elem.clone()))
                .collect(),
            alignment: Alignment::Justified,
        })
    }

    pub fn elem(text: &str) -> Layout {
        Layout::Element(Element {
            text: text.to_string(),
            font: Font {
                name: String::from("Arial"),
                size: 12,
            },
            alignment: Alignment::Left,
        })
    }
}

impl Layout {
    pub fn instantiate(
        &self,
        section: &HashMap<String, ItemContent>,
        schema: &Vec<Field>,
    ) -> Layout {
        match self {
            Layout::Stack(stacks) => {
                let mut instantiated_stacks = Vec::new();
                for stack in stacks {
                    instantiated_stacks.push(stack.instantiate(section, schema));
                }
                Layout::Stack(instantiated_stacks)
            }
            Layout::Grid(grid) => {
                let mut instantiated_elements = Vec::new();
                for element in &grid.elements {
                    let instantiated_element = element.instantiate(section, schema);
                    instantiated_elements.push(instantiated_element);
                }
                Layout::Grid(LayoutGrid {
                    elements: instantiated_elements,
                    alignment: grid.alignment.clone(),
                })
            }
            Layout::Element(elem) => Layout::Element(Element {
                text: section
                    .get(&elem.text)
                    .unwrap_or(&ItemContent::String(elem.text.clone()))
                    .to_string(),
                font: elem.font.clone(),
                alignment: elem.alignment.clone(),
            }),
        }
    }

    pub fn render(
        layout_schemas: Vec<LayoutSchema>,
        resume_data: ResumeData,
        data_schemas: Vec<DataSchema>,
        filepath: &Path,
    ) -> std::io::Result<()> {
        let mut contents = String::new();

        for section in resume_data.sections {
            // Render Section Header
            // 1. Find the layout schema for the section
            let Some(layout_schema) = layout_schemas
                .iter()
                .find(|&s| s.schema_name == section.layout_schema)
            else {
                return Err(Error::new(ErrorKind::Other, format!("Layout not found for {}", section.layout_schema)));
            };
            // 2. Find the data schema for the section
            let data_schema = data_schemas
                .iter()
                .find(|&s| s.name == section.data_schema)
                .unwrap();
            // 3. Render the header

            let instantiated_header = layout_schema
                .header_layout_schema
                .instantiate(&section.data, &data_schema.header_schema);
            let header = instantiated_header.blueprint();

            contents.push_str(&header);

            // Render Section Items
            for item in section.items {
                // 1. Find the layout schema for the section
                let layout_schema = layout_schemas
                    .iter()
                    .find(|&s| s.schema_name == section.layout_schema)
                    .unwrap();
                // 2. Find the data schema for the section
                let data_schema = data_schemas
                    .iter()
                    .find(|&s| s.name == section.data_schema)
                    .unwrap();
                // 3. Render the item

                let instantiated_item = layout_schema
                    .item_layout_schema
                    .instantiate(&item, &data_schema.item_schema);
                let item = instantiated_item.blueprint();
                contents.push_str(&item);
            }
        }

        fs::write(filepath, contents)
    }
}

impl Layout {
    fn compute_height(&self) -> u32 {
        match self {
            Layout::Stack(stacks) => stacks.iter().map(|e| e.compute_height()).sum::<u32>(),
            Layout::Grid(grid) => {
                grid.elements
                    .iter()
                    .map(|e| e.margin.top + e.margin.bottom + e.element.compute_height())
                    .max()
                    .unwrap_or_default()
            }
            Layout::Element(_) => 1,
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

            }
            Layout::Grid(grid) => {
                grid.elements
                    .iter()
                    .map(|e| {
                        e.margin.left + e.margin.right + e.element.compute_total_elements_width()
                    })
                    .sum::<u32>()

            }
            Layout::Element(elem) => elem.text.len() as u32,
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
        mut current_box: SpatialBox,
    ) {

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
                        let per_element_spacing = if (grid.elements.len() > 1) {
                            total_spacing / (grid.elements.len() - 1) as u32
                        } else {
                            0
                        };
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LayoutGrid {
    pub elements: Vec<GridElement>,
    pub alignment: Alignment,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GridElement {
    pub min_width: Option<u32>,
    pub max_width: Option<u32>,
    pub default_width: Option<u32>,
    pub margin: Margin,
    pub element: Layout,
}

impl GridElement {
    pub fn default(element: Layout) -> GridElement {
        GridElement {
            min_width: None,
            max_width: Some(100),
            default_width: Some(70),
            margin: Margin::default(),
            element,
        }
    }
}

impl GridElement {
    pub fn instantiate(
        &self,
        section: &HashMap<String, ItemContent>,
        schema: &Vec<Field>,
    ) -> GridElement {
        GridElement {
            min_width: self.min_width,
            max_width: self.max_width,
            default_width: self.default_width,
            margin: self.margin.clone(),
            element: self.element.instantiate(section, schema),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Element {
    pub text: String,
    pub font: Font,
    pub alignment: Alignment,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
            Layout::Grid(LayoutGrid {
                elements: vec![
                    GridElement {
                        min_width: None,
                        max_width: None,
                        default_width: None,
                        margin: Margin {
                            top: 3,
                            bottom: 0,
                            left: 20,
                            right: 0,
                        },
                        element: Layout::Element(Element {
                            text: String::from("A"),
                            font: Font {
                                name: String::from("Arial"),
                                size: 12,
                            },
                            alignment: Alignment::Left,
                        }),
                    },
                    GridElement {
                        min_width: None,
                        max_width: None,
                        default_width: None,
                        margin: Margin {
                            top: 0,
                            bottom: 2,
                            left: 0,
                            right: 20,
                        },
                        element: Layout::Element(Element {
                            text: String::from("B"),
                            font: Font {
                                name: String::from("Arial"),
                                size: 12,
                            },
                            alignment: Alignment::Left,
                        }),
                    },
                ],
                alignment: Alignment::Justified,
            }),
            Layout::Grid(LayoutGrid {
                elements: vec![
                    GridElement {
                        min_width: None,
                        max_width: None,
                        default_width: None,
                        margin: Margin {
                            top: 3,
                            bottom: 0,
                            left: 20,
                            right: 0,
                        },
                        element: Layout::Element(Element {
                            text: String::from("A"),
                            font: Font {
                                name: String::from("Arial"),
                                size: 12,
                            },
                            alignment: Alignment::Left,
                        }),
                    },
                    GridElement {
                        min_width: None,
                        max_width: None,
                        default_width: None,
                        margin: Margin {
                            top: 0,
                            bottom: 2,
                            left: 0,
                            right: 20,
                        },
                        element: Layout::Element(Element {
                            text: String::from("B"),
                            font: Font {
                                name: String::from("Arial"),
                                size: 12,
                            },
                            alignment: Alignment::Left,
                        }),
                    },
                ],
                alignment: Alignment::Left,
            }),
            Layout::Grid(LayoutGrid {
                elements: vec![
                    GridElement {
                        min_width: None,
                        max_width: None,
                        default_width: None,
                        margin: Margin {
                            top: 3,
                            bottom: 0,
                            left: 20,
                            right: 0,
                        },
                        element: Layout::Element(Element {
                            text: String::from("A"),
                            font: Font {
                                name: String::from("Arial"),
                                size: 12,
                            },
                            alignment: Alignment::Left,
                        }),
                    },
                    GridElement {
                        min_width: None,
                        max_width: None,
                        default_width: None,
                        margin: Margin {
                            top: 0,
                            bottom: 2,
                            left: 0,
                            right: 20,
                        },
                        element: Layout::Element(Element {
                            text: String::from("B"),
                            font: Font {
                                name: String::from("Arial"),
                                size: 12,
                            },
                            alignment: Alignment::Left,
                        }),
                    },
                ],
                alignment: Alignment::Right,
            }),
            Layout::Grid(LayoutGrid {
                elements: vec![
                    GridElement {
                        min_width: None,
                        max_width: None,
                        default_width: None,
                        margin: Margin {
                            top: 3,
                            bottom: 0,
                            left: 20,
                            right: 0,
                        },
                        element: Layout::Element(Element {
                            text: String::from("A"),
                            font: Font {
                                name: String::from("Arial"),
                                size: 12,
                            },
                            alignment: Alignment::Left,
                        }),
                    },
                    GridElement {
                        min_width: None,
                        max_width: None,
                        default_width: None,
                        margin: Margin {
                            top: 0,
                            bottom: 2,
                            left: 0,
                            right: 20,
                        },
                        element: Layout::Element(Element {
                            text: String::from("B"),
                            font: Font {
                                name: String::from("Arial"),
                                size: 12,
                            },
                            alignment: Alignment::Left,
                        }),
                    },
                ],
                alignment: Alignment::Center,
            }),
        ]);
    }
}

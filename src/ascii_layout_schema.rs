use core::num;
use std::borrow::BorrowMut;
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

impl LayoutSchema {
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
            contents.push_str(
                &layout_schema
                    .header_layout_schema
                    .instantiate(&section.data, &data_schema.header_schema)
                    .render(),
            );

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
                contents.push_str(
                    &layout_schema
                        .item_layout_schema
                        .instantiate(&item, &data_schema.item_schema)
                        .render(),
                );
            }
        }

        fs::write(filepath, contents)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Layout {
    Stack(Stack),
    Container(Container),
}

impl Layout {
    pub fn mk_row(layouts: Vec<Layout>) -> Layout {
        Layout::Container(Container {
            width: Width::Fixed(layouts.iter().map(|l| l.get_width()).sum()),
            inner: ContainerInner::Grid(layouts),
            alignment: Alignment::default(),
            margin: Margin::default(),
        })
    }

    pub fn mk_stack(layouts: Vec<Layout>) -> Layout {
        Layout::Stack(Stack {
            layouts: layouts
                .iter()
                .map(|layout| Container {
                    width: Width::Fixed(layout.get_width()),
                    inner: ContainerInner::Grid(vec![layout.clone()]),
                    alignment: Alignment::default(),
                    margin: Margin::default(),
                })
                .collect(),
        })
    }

    pub fn mk_text(text: String) -> Layout {
        Layout::Container(Container::text_container(text))
    }

    pub fn mk_ref(text: String) -> Layout {
        Layout::Container(Container::ref_container(text))
    }
    
    pub fn wrap_in_container(&self) -> Layout {
        Layout::Container(Container {
            width: Width::Fill,
            inner: ContainerInner::Grid(vec![self.clone()]),
            alignment: Alignment::default(),
            margin: Margin::default(),
        })
    }

    pub fn with_width(&self, width: u32) -> Layout {
        match self {
            Layout::Stack(stack) => self.wrap_in_container().with_width(width),
            Layout::Container(container) => Layout::Container(Container {
                width: Width::Fixed(width),
                inner: container.inner.clone(),
                alignment: container.alignment,
                margin: container.margin,
            }),
        }
    }


    pub fn with_margin(&self, margin: Margin) -> Layout {
        match self {
            Layout::Stack(stack) => self.wrap_in_container().with_margin(margin),
            Layout::Container(container) => Layout::Container(Container {
                width: Width::Fixed(container.get_width() + margin.left + margin.right),
                inner: container.inner.clone(),
                alignment: container.alignment,
                margin: margin,
            }),
        }
    }

    pub fn with_alignment(&self, alignment: Alignment) -> Layout {
        match self {
            Layout::Stack(stack) => self.wrap_in_container().with_alignment(alignment),
            Layout::Container(container) => Layout::Container(Container {
                width: container.width,
                inner: container.inner.clone(),
                alignment: alignment,
                margin: container.margin,
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Width {
    Fixed(u32),
    Fill,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Container {
    pub inner: ContainerInner,
    pub width: Width,
    pub alignment: Alignment,
    pub margin: Margin,
}

impl Container {
    pub fn get_width(&self) -> u32 {
        match self.width {
            Width::Fixed(w) => w,
            Width::Fill => self.inner.get_width(),
        }
    }

    pub fn text_container(text: String) -> Container {
        Container {
            width: Width::Fixed(text.len() as u32),
            inner: ContainerInner::Element(Element::Text(text)),
            alignment: Alignment::Left,
            margin: Margin::default(),
        }
    }

    pub fn text_container_with_margin(text: String, margin: Margin) -> Container {
        Container {
            width: Width::Fixed(text.len() as u32 + margin.left + margin.right),
            inner: ContainerInner::Element(Element::Text(text)),
            alignment: Alignment::Left,
            margin: margin,
        }
    }

    pub fn ref_container(text: String) -> Container {
        Container {
            inner: ContainerInner::Element(Element::Ref(text)),
            width: Width::Fill,
            alignment: Alignment::Left,
            margin: Margin::default(),
        }
    }

    pub fn ref_container_with_margin(text: String, margin: Margin) -> Container {
        Container {
            inner: ContainerInner::Element(Element::Ref(text)),
            width: Width::Fill,
            alignment: Alignment::Left,
            margin: margin,
        }
    }
}

impl Container {
    pub fn instantiate(
        &self,
        section: &HashMap<String, ItemContent>,
        schema: &Vec<Field>,
    ) -> Container {
        match &self.inner {
            ContainerInner::Grid(layouts) => Container {
                inner: ContainerInner::Grid(
                    layouts
                        .iter()
                        .map(|l| l.instantiate(section, schema))
                        .collect::<Vec<Layout>>(),
                ),
                width: self.width,
                alignment: self.alignment,
                margin: self.margin,
            },
            ContainerInner::Element(Element::Text(text)) => Container {
                inner: ContainerInner::Element(Element::Text(text.clone())),
                width: self.width,
                alignment: self.alignment,
                margin: self.margin,
            },
            ContainerInner::Element(Element::Ref(name)) => {
                if let Some(text) = section.get(name) {
                    Container {
                        inner: ContainerInner::Element(Element::Text(text.to_string())),
                        width: self.width,
                        alignment: self.alignment,
                        margin: self.margin,
                    }
                } else {
                    Container {
                        inner: ContainerInner::Grid(vec![]),
                        width: Width::Fixed(0),
                        alignment: self.alignment,
                        margin: Margin::default(),
                    }
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ContainerInner {
    Grid(Vec<Layout>),
    Element(Element),
}

impl ContainerInner {
    pub fn get_width(&self) -> u32 {
        match self {
            ContainerInner::Grid(layouts) => layouts.iter().map(|l| l.get_width()).sum(),
            ContainerInner::Element(e) => e.text().len() as u32,
        }
    }
}

impl Layout {
    pub fn text_container(text: String, width: Width) -> Layout {
        Layout::Container(Container {
            inner: ContainerInner::Element(Element::Text(text)),
            width: width,
            alignment: Alignment::Left,
            margin: Margin::default(),
        })
    }

    pub fn text_container_with_margin(text: String, width: Width, margin: Margin) -> Layout {
        Layout::Container(Container {
            inner: ContainerInner::Element(Element::Text(text)),
            width: width,
            alignment: Alignment::Left,
            margin: margin,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stack {
    pub layouts: Vec<Container>,
}

impl Stack {
    pub fn get_width(&self) -> u32 {
        self.layouts.iter().map(|c| c.get_width()).max().unwrap()
    }
}

impl Layout {
    pub fn instantiate(
        &self,
        section: &HashMap<String, ItemContent>,
        schema: &Vec<Field>,
    ) -> Layout {
        match self {
            Layout::Stack(s) => Layout::Stack(Stack {
                layouts: s
                    .layouts
                    .iter()
                    .map(|c| c.instantiate(section, schema))
                    .collect(),
            }),
            Layout::Container(c) => Layout::Container(c.instantiate(section, schema)),
        }
    }
}

impl Layout {
    fn get_width(&self) -> u32 {
        match self {
            Layout::Container(c) => c.get_width(),
            Layout::Stack(s) => s.get_width(),
        }
    }

    pub fn render(&self) -> String {
        let mut layout_map: HashMap<Point, char> = HashMap::new();
        let top_left: Point = Point::new(0, 0);
        let width = self.get_width();

        self.compute_layout_map(&mut layout_map, top_left, width);

        let height = layout_map.keys().map(|p| p.y).max().unwrap_or(0) + 1;

        let mut output = String::new();

        // output.push(' ');
        // for x in 0..width {
        //     output.push((x % 10).to_string().chars().next().unwrap());
        // }
        // output.push('\n');
        for y in 0..height {
            // output.push((y % 10).to_string().chars().next().unwrap());
            for x in 0..width {
                let point = Point::new(x, y);
                let c = layout_map.get(&point).unwrap_or(&' ');
                output.push(*c);
            }
            output.push('\n');
        }

        output
    }

    fn compute_layout_map(
        &self,
        mut blueprintmap: &mut HashMap<Point, char>,
        top_left: Point,
        width: u32,
    ) -> u32 {
        match self {
            Layout::Stack(stack) => {
                let mut top_left = top_left;
                let mut depth = top_left.y;
                for element in stack.layouts.iter() {
                    depth = Layout::Container(element.clone()).compute_layout_map(
                        &mut blueprintmap,
                        top_left,
                        width,
                    );

                    top_left = top_left.move_y_to(depth);
                }
                depth
            }
            Layout::Container(container) => {
                if container.get_width() == 0 {
                    return top_left.y;
                }
                if container.get_width() <= container.margin.left + container.margin.right {
                    panic!("Container width is too small or margins are too large");
                }

                let container_width =
                    container.get_width() - container.margin.left - container.margin.right;

                if container_width < (&container.inner).get_width() {
                    match &container.inner {
                        ContainerInner::Grid(layouts) => {
                            todo!()
                        }
                        ContainerInner::Element(element) => {
                            let total_length = element.text().len() as u32;

                            let number_of_lines =
                                (total_length as f32 / container_width as f32).ceil() as u32;

                            let mut layouts = Vec::new();

                            for i in 0..number_of_lines {
                                let start = i * container_width;
                                let text = element
                                    .text()
                                    .chars()
                                    .skip(start as usize)
                                    .take(container_width as usize)
                                    .collect::<String>();

                                let mut text_container = Container {
                                    inner: ContainerInner::Element(Element::Text(text)),
                                    width: Width::Fixed(container_width),
                                    alignment: container.alignment.clone(),
                                    margin: container.margin.clone(),
                                };

                                text_container.margin = if i == 0 {
                                    Margin::new(
                                        container.margin.top,
                                        0,
                                        container.margin.left,
                                        container.margin.right,
                                    )
                                } else if i == number_of_lines - 1 {
                                    Margin::new(
                                        0,
                                        container.margin.bottom,
                                        container.margin.left,
                                        container.margin.right,
                                    )
                                } else {
                                    Margin::new(0, 0, container.margin.left, container.margin.right)
                                };
                                layouts.push(text_container);
                            }

                            Layout::Stack(Stack { layouts }).compute_layout_map(
                                &mut blueprintmap,
                                top_left,
                                width,
                            )
                        }
                    }
                } else {
                    let space = container_width - (&container.inner).get_width();
                    match container.alignment {
                        Alignment::Left => {
                            let top_left = top_left
                                .move_x_by(container.margin.left as i32)
                                .move_y_by(container.margin.top as i32);

                            match &container.inner {
                                ContainerInner::Grid(ref layouts) => {
                                    let mut top_left = top_left;
                                    let mut max_depth = top_left.y;
                                    for element in layouts.iter() {
                                        let depth = element.compute_layout_map(
                                            &mut blueprintmap,
                                            top_left,
                                            element.get_width(),
                                        );
                                        max_depth = u32::max(max_depth, depth);
                                        top_left = top_left.move_x_by(element.get_width() as i32);
                                    }

                                    max_depth
                                }
                                ContainerInner::Element(element) => {
                                    let top_right = top_left.move_x_by(container_width as i32);

                                    for x in top_left.x..top_right.x {
                                        if let Some(c) =
                                            element.text().chars().nth((x - top_left.x) as usize)
                                        {
                                            blueprintmap.insert(Point::new(x, top_left.y), c);
                                        }
                                    }

                                    top_right.y + container.margin.bottom + 1
                                }
                            }
                        }
                        Alignment::Center => {
                            let top_left = top_left
                                .move_x_by((space / 2 + container.margin.left) as i32)
                                .move_y_by(container.margin.top as i32);

                            match &container.inner {
                                ContainerInner::Grid(ref layouts) => {
                                    let mut top_left = top_left;
                                    let mut max_depth = top_left.y;
                                    for element in layouts.iter() {
                                        let depth = element.compute_layout_map(
                                            &mut blueprintmap,
                                            top_left,
                                            element.get_width(),
                                        );
                                        max_depth = u32::max(max_depth, depth);
                                        top_left = top_left.move_x_by(element.get_width() as i32);
                                    }

                                    max_depth
                                }
                                ContainerInner::Element(element) => {
                                    let top_right = top_left.move_x_by(container_width as i32);

                                    for x in top_left.x..top_right.x {
                                        if let Some(c) =
                                            element.text().chars().nth((x - top_left.x) as usize)
                                        {
                                            blueprintmap.insert(Point::new(x, top_left.y), c);
                                        }
                                    }

                                    top_right.y + container.margin.bottom + 1
                                }
                            }
                        }

                        Alignment::Right => {
                            let top_left = top_left
                                .move_x_by((space + container.margin.left) as i32)
                                .move_y_by(container.margin.top as i32);

                            match &container.inner {
                                ContainerInner::Grid(ref layouts) => {
                                    let mut top_left = top_left;
                                    let mut max_depth = top_left.y;
                                    for element in layouts.iter() {
                                        let depth = element.compute_layout_map(
                                            &mut blueprintmap,
                                            top_left,
                                            element.get_width(),
                                        );
                                        max_depth = u32::max(max_depth, depth);
                                        top_left = top_left.move_x_by(element.get_width() as i32);
                                    }

                                    max_depth
                                }
                                ContainerInner::Element(element) => {
                                    let top_right = top_left.move_x_by(container_width as i32);

                                    for x in top_left.x..top_right.x {
                                        if let Some(c) =
                                            element.text().chars().nth((x - top_left.x) as usize)
                                        {
                                            blueprintmap.insert(Point::new(x, top_left.y), c);
                                        }
                                    }

                                    top_right.y + container.margin.bottom + 1
                                }
                            }
                        }
                        Alignment::Justified => todo!(),
                    }
                }
            }
        }
    }
}

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Element {
    Ref(String),
    Text(String),
}

impl Element {
    pub fn text(&self) -> String {
        match self {
            Element::Ref(s) | Element::Text(s) => s.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_blueprint() {}
}

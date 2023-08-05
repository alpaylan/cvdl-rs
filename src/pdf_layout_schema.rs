use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::io::Write;

use printpdf::IndirectFontRef;
use printpdf::Mm;
use printpdf::PdfDocument;
use printpdf::PdfLayerReference;
use serde::{Deserialize, Serialize};

use std::{io::ErrorKind, path::Path};

use printpdf;
use std::fs;
use std::io::BufWriter;
use std::io::Error;

use crate::alignment::Alignment;
use crate::data_schema::{DataSchema, Field};
use crate::margin::Margin;
use crate::point::Point;
use crate::resume_data::{ItemContent, ResumeData};

use rusttype::{point, Font, Scale};
use image::{DynamicImage, Rgba};


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
        let (doc, page1, layer1) =
            PdfDocument::new("PDF_Document_title", Mm(612.0), Mm(792.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);

        let font = doc
            .add_external_font(fs::File::open("assets/Exo/static/Exo-Medium.ttf").unwrap())
            .unwrap();

        let mut height = 0;

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

            height = layout_schema
                .header_layout_schema
                .instantiate(&section.data, &data_schema.header_schema)
                .render(&current_layer, &font, 10);

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
                height = layout_schema
                    .item_layout_schema
                    .instantiate(&item, &data_schema.item_schema)
                    .render(&current_layer, &font, height);
            }
        }

        doc.save(&mut BufWriter::new(fs::File::create(filepath).unwrap()))
            .unwrap();

        // fs::write(filepath, contents)
        Ok(())
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
            width: Width::Fixed(get_width_and_height(text.clone()).1),
            inner: ContainerInner::Element(Element::Text(text)),
            alignment: Alignment::Left,
            margin: Margin::default(),
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
            ContainerInner::Element(e) => get_width_and_height(e.text().clone()).1,
        }
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

    pub fn render(
        &self,
        pdf_layer: &PdfLayerReference,
        font: &IndirectFontRef,
        height: u32,
    ) -> u32 {
        let mut layout_map: HashMap<Point, char> = HashMap::new();
        let top_left: Point = Point::new(0, height);
        let width = self.get_width();

        self.compute_layout_map(&mut layout_map, top_left, width, pdf_layer, font)
    }

    fn compute_layout_map(
        &self,
        mut blueprintmap: &mut HashMap<Point, char>,
        top_left: Point,
        width: u32,
        current_layer: &PdfLayerReference,
        font: &IndirectFontRef,
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
                        current_layer,
                        font,
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
                            let total_length = get_width_and_height(element.text().clone()).1;

                            let number_of_lines =
                                (total_length as f32 / container_width as f32).ceil() as u32;

                            
                            let mut layouts = Vec::new();
                            
                            let mut start = 0;
                            for i in 0..number_of_lines {
                                let mut len = element.text().len() - start;
                                while(get_width_and_height(element.text().as_str()[start..(start + len)].to_string()).1 > container_width) {
                                    len -= 1;
                                }
                                let text = element.text().as_str()[start..(start + len)].to_string();
                                start = start + len;
                                let mut text_container = Container {
                                    inner: ContainerInner::Element(Element::Text(text)),
                                    width: Width::Fixed(container_width),
                                    alignment: container.alignment.clone(),
                                    margin: container.margin.clone(),
                                };

                                text_container.margin = if i == 0 {
                                    container.margin.with_bottom(0)
                                } else if i == number_of_lines - 1 {
                                    container.margin.with_top(0)
                                } else {
                                    container.margin.with_bottom(0).with_top(0)
                                };
                                layouts.push(text_container);
                            }

                            Layout::Stack(Stack { layouts }).compute_layout_map(
                                &mut blueprintmap,
                                top_left,
                                width,
                                current_layer,
                                font,
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
                                            current_layer,
                                            font,
                                        );
                                        max_depth = u32::max(max_depth, depth);
                                        top_left = top_left.move_x_by(element.get_width() as i32);
                                    }

                                    max_depth
                                }
                                ContainerInner::Element(element) => {
                                    let top_right = top_left.move_x_by(container_width as i32);


                                    println!(
                                        "Drawing text {:?} at {}x{}",
                                        element.text(),
                                        top_left.x,
                                        792 - top_left.y
                                    );

                                    let height = get_width_and_height(element.text().clone()).0;

                                    current_layer.use_text(
                                        element.text().clone(),
                                        24.0,
                                        Mm(top_left.x.into()),
                                        Mm((789 - (top_left.y + height) ).into()),
                                        &font,
                                    );
                                    top_right.y + container.margin.bottom + height
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
                                            current_layer,
                                            font,
                                        );
                                        max_depth = u32::max(max_depth, depth);
                                        top_left = top_left.move_x_by(element.get_width() as i32);
                                    }

                                    max_depth
                                }
                                ContainerInner::Element(element) => {
                                    let top_right = top_left.move_x_by(container_width as i32);

                                    println!(
                                        "Drawing text {} at {}x{}",
                                        element.text(),
                                        top_left.x,
                                        792 - top_left.y
                                    );

                                    current_layer.use_text(
                                        element.text().clone(),
                                        24.0,
                                        Mm(top_left.x.into()),
                                        Mm((789 - top_left.y * 4).into()),
                                        &font,
                                    );

                                    top_right.y + container.margin.bottom + get_width_and_height(element.text().clone()).0
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
                                            current_layer,
                                            font,
                                        );
                                        max_depth = u32::max(max_depth, depth);
                                        top_left = top_left.move_x_by(element.get_width() as i32);
                                    }

                                    max_depth
                                }
                                ContainerInner::Element(element) => {
                                    let top_right = top_left.move_x_by(container_width as i32);

                                    println!(
                                        "Drawing text {} at {}x{}",
                                        element.text(),
                                        top_left.x,
                                        792 - top_left.y
                                    );

                                    current_layer.use_text(
                                        element.text().clone(),
                                        24.0,
                                        Mm(top_left.x.into()),
                                        Mm((789 - top_left.y * 4).into()),
                                        &font,
                                    );

                                    top_right.y + container.margin.bottom + get_width_and_height(element.text().clone()).0
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


pub fn get_width_and_height(text: String) -> (u32, u32) {
    
    let font_data = include_bytes!("../assets/Exo/static/Exo-Medium.ttf");
    // This only succeeds if collection consists of one font
    let _font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");

    // The font size to use
    let scale = Scale::uniform(12.0);

    // The text to render
    let v_metrics = _font.v_metrics(scale);

    // layout the glyphs in a line with 20 pixels padding
    let glyphs: Vec<_> = _font
        .layout(text.trim(), scale, point(0_f32, v_metrics.ascent))
        .collect();

    // work out the layout size
    let glyphs_height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;
    let glyphs_width = {
        let min_x = glyphs
            .first()
            .map(|g| g.pixel_bounding_box().unwrap().min.x)
            .unwrap();
        let max_x = glyphs
            .last()
            .map(|g| g.pixel_bounding_box().unwrap().max.x)
            .unwrap();
        (max_x - min_x) as u32
    };

    println!("glyphs_height: {}, glyphs_width: {}", glyphs_height, glyphs_width);

    (glyphs_height, glyphs_width)
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

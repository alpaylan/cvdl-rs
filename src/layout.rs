use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::alignment::Alignment;
use crate::basic_layout::BasicLayout;
use crate::container::Container;
use crate::document::DocumentDefinition;
use crate::element::Element;
use crate::font::FontDict;
use crate::margin::Margin;
use crate::point::Point;
use crate::resume_data::ItemContent;
use crate::spatial_box::SpatialBox;
use crate::width::Width;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Layout {
    Stack(Container),
    FrozenRow(Container),
    FlexRow(Container),
    Text(Element),
    Ref(Element),
}

// todo: use this
impl Into<BasicLayout> for Layout {
    fn into(self) -> BasicLayout {
        match self {
            Layout::Stack(_container) => todo!(),
            Layout::FrozenRow(_) => unreachable!("FrozenRow should be converted to FlexRow"),
            Layout::FlexRow(_container) => todo!(),
            Layout::Text(_element) => todo!(),
            Layout::Ref(_) => unreachable!("Ref should be converted to Text"),
        }
    }
}

impl Layout {
    pub fn new_stack(container: Container) -> Layout {
        Layout::Stack(container)
    }

    pub fn new_frozen_row(container: Container) -> Layout {
        Layout::FrozenRow(container)
    }

    pub fn new_flex_row(container: Container) -> Layout {
        Layout::FlexRow(container)
    }

    pub fn new_text(element: Element) -> Layout {
        Layout::Text(element)
    }

    pub fn new_ref(element: Element) -> Layout {
        Layout::Ref(element)
    }
}

impl Layout {
    pub fn type_(&self) -> String {
        match self {
            Layout::Stack(_) => "stack".to_string(),
            Layout::FrozenRow(_) => "frozen_row".to_string(),
            Layout::FlexRow(_) => "flex_row".to_string(),
            Layout::Text(_) => "text".to_string(),
            Layout::Ref(_) => "ref".to_string(),
        }
    }

    pub fn width(&self) -> Width {
        match self {
            Layout::Stack(container)
            | Layout::FrozenRow(container)
            | Layout::FlexRow(container) => container.width,
            Layout::Text(element) | Layout::Ref(element) => element.width,
        }
    }

    pub fn margin(&self) -> Margin {
        match self {
            Layout::Stack(container)
            | Layout::FrozenRow(container)
            | Layout::FlexRow(container) => container.margin,
            Layout::Text(element) | Layout::Ref(element) => element.margin,
        }
    }

    pub fn alignment(&self) -> Alignment {
        match self {
            Layout::Stack(container)
            | Layout::FrozenRow(container)
            | Layout::FlexRow(container) => container.alignment,
            Layout::Text(element) | Layout::Ref(element) => element.alignment,
        }
    }

    pub fn with_margin(&self, margin: Margin) -> Layout {
        match self {
            Layout::Stack(container) => Layout::new_stack(container.with_margin(margin)),
            Layout::FrozenRow(container) => Layout::new_frozen_row(container.with_margin(margin)),
            Layout::FlexRow(container) => Layout::new_flex_row(container.with_margin(margin)),
            Layout::Text(element) => Layout::new_text(element.with_margin(margin)),
            Layout::Ref(element) => Layout::new_ref(element.with_margin(margin)),
        }
    }

    pub fn with_alignment(&self, alignment: Alignment) -> Layout {
        match self {
            Layout::Stack(container) => Layout::new_stack(container.with_alignment(alignment)),
            Layout::FrozenRow(container) => {
                Layout::new_frozen_row(container.with_alignment(alignment))
            }
            Layout::FlexRow(container) => Layout::new_flex_row(container.with_alignment(alignment)),
            Layout::Text(element) => Layout::new_text(element.with_alignment(alignment)),
            Layout::Ref(element) => Layout::new_ref(element.with_alignment(alignment)),
        }
    }

    pub fn is_instantiated(&self) -> bool {
        match self {
            Layout::Stack(c) | Layout::FrozenRow(c) | Layout::FlexRow(c) => {
                c.elements.iter().all(|e| e.is_instantiated())
            }
            Layout::Text(_) => true,
            Layout::Ref(_) => false,
        }
    }

    pub fn instantiate(&self, section: &HashMap<String, ItemContent>) -> Layout {
        match self {
            Layout::Stack(c) => Layout::new_stack(c.instantiate(section)),
            Layout::FrozenRow(c) => Layout::new_frozen_row(c.instantiate(section)),
            Layout::FlexRow(c) => Layout::new_flex_row(c.instantiate(section)),
            Layout::Text(e) => Layout::new_text(e.clone()),
            Layout::Ref(e) => Layout::instantiate_ref_element(e.clone(), section),
        }
    }

    pub fn instantiate_ref_element(
        element: Element,
        section: &HashMap<String, ItemContent>,
    ) -> Layout {
        if let Some(text) = section.get(&element.item) {
            Layout::Text(element.with_item(text.to_string()))
        } else {
            Layout::Stack(Container::empty_container())
        }
    }

    pub fn propagate_widths(&self) -> Layout {
        if let Width::Fixed(w) = self.width() {
            self.bound_width(w)
        } else {
            panic!("Cannot fix width of layout with non-fixed width");
        }
    }

    pub fn bound_width(&self, width: f32) -> Layout {
        let bound = if self.width().is_fixed() && self.width().get_fixed().unwrap() <= width {
            self.width().get_fixed().unwrap()
        } else {
            width
        };

        match self {
            Layout::Stack(c) => Layout::new_stack(c.bound_width(bound)),
            Layout::FrozenRow(c) => Layout::new_frozen_row(c.bound_width(bound)),
            Layout::FlexRow(c) => Layout::new_flex_row(c.bound_width(bound)),
            Layout::Text(e) => Layout::new_text(e.bound_width(bound)),
            Layout::Ref(_) => unreachable!("Cannot propagate widths of uninstantiated layout"),
        }
    }

    pub fn is_bounded(&self) -> bool {
        match self {
            Layout::Stack(c) | Layout::FrozenRow(c) | Layout::FlexRow(c) => {
                if c.width.is_fixed() {
                    c.elements.iter().all(|e| e.is_bounded())
                } else {
                    false
                }
            }
            Layout::Text(e) => e.width.is_fixed(),
            Layout::Ref(_) => unreachable!("Cannot check if uninstantiated layout is bounded"),
        }
    }

    pub fn scale_width(&self, document_width: u32) -> Layout {
        match self {
            Layout::Stack(c) => Layout::new_stack(c.scale_width(document_width)),
            Layout::FrozenRow(c) => Layout::new_frozen_row(c.scale_width(document_width)),
            Layout::FlexRow(c) => Layout::new_flex_row(c.scale_width(document_width)),
            Layout::Text(e) => Layout::new_text(e.scale_width(document_width)),
            Layout::Ref(_) => unreachable!("Cannot scale width of uninstantiated layout"),
        }
    }

    pub fn normalize(&self, document: &DocumentDefinition) -> Layout {
        if !self.is_instantiated() {
            panic!("Cannot normalize uninstantiated layout");
        };

        if !self.is_bounded() {
            panic!("Cannot normalize unbounded layout");
        };

        let scaled_layout = self.scale_width(document.width);

        let font_filled_layout = scaled_layout.fill_fonts(&document.font_dict);

        let broken_layout = font_filled_layout.break_lines(&document.font_dict);

        broken_layout
    }

    pub fn fill_fonts(&self, font_dict: &FontDict) -> Layout {
        match self {
            Layout::Stack(c) => Layout::new_stack(c.fill_fonts(font_dict)),
            Layout::FrozenRow(c) => Layout::new_frozen_row(c.fill_fonts(font_dict)),
            Layout::FlexRow(c) => Layout::new_flex_row(c.fill_fonts(font_dict)),
            Layout::Text(e) => Layout::new_text(e.fill_fonts(font_dict)),
            Layout::Ref(_) => unreachable!("Cannot fill fonts of uninstantiated layout"),
        }
    }

    pub fn break_lines(&self, font_dict: &FontDict) -> Layout {
        match self {
            Layout::Stack(c) => {
                let new_stack = Layout::new_stack(
                    c.with_elements(
                        c.elements
                            .iter()
                            .map(|e| e.break_lines(font_dict))
                            .collect(),
                    ),
                );
                new_stack
            }
            Layout::FrozenRow(c) => {
                let total_width = c
                    .elements
                    .iter()
                    .map(|e| e.width().get_fixed_unchecked())
                    .sum::<f32>();
                if total_width > self.width().get_fixed_unchecked() {
                    panic!(
                        "Cannot break lines of frozen row with width {:?} and total width {}",
                        self.width(),
                        total_width
                    );
                } else {
                    Layout::new_flex_row(Container {
                        elements: c
                            .elements
                            .iter()
                            .map(|e| e.break_lines(font_dict))
                            .collect(),
                        margin: c.margin,
                        alignment: c.alignment,
                        width: c.width,
                    })
                }
            }
            Layout::FlexRow(c) => {
                let lines: Vec<Container> = c.break_lines(font_dict);
                Layout::new_stack(
                    c.with_elements(lines.into_iter().map(|c| Layout::FlexRow(c)).collect()),
                )
            }
            Layout::Text(e) => {
                let lines: Vec<Element> = e.break_lines(font_dict);
                Layout::new_stack(
                    Container::empty_container()
                        .with_elements(lines.into_iter().map(|e| Layout::new_text(e)).collect())
                        .with_alignment(e.alignment)
                        .with_margin(e.margin)
                        .with_width(e.width),
                )
            }
            Layout::Ref(_) => unreachable!("Cannot break lines of uninstantiated layout"),
        }
    }
}

impl Layout {
    pub fn compute_boxes(
        &self,
        height_offset: u32,
        font_dict: &FontDict,
    ) -> (u32, Vec<(SpatialBox, Element)>) {
        let mut textbox_positions: Vec<(SpatialBox, Element)> = Vec::new();
        let top_left: Point = Point::new(0, height_offset);
        let depth = self.compute_textbox_positions(&mut textbox_positions, top_left, font_dict);

        (depth, textbox_positions)
    }

    fn compute_textbox_positions(
        &self,
        mut textbox_positions: &mut Vec<(SpatialBox, Element)>,
        top_left: Point,
        font_dict: &FontDict,
    ) -> u32 {
        match self {
            Layout::Stack(c) => {
                let mut top_left = top_left;
                let mut depth = top_left.y;
                for element in c.elements.iter() {
                    depth =
                        element.compute_textbox_positions(textbox_positions, top_left, font_dict);
                    top_left = top_left.move_y_to(depth);
                }
                depth
            }
            Layout::FlexRow(c) => {
                let (top_left, per_elem_space) = match c.alignment {
                    Alignment::Left => (top_left, 0.0),
                    Alignment::Center => (
                        top_left.move_x_by(
                            ((c.width.get_fixed_unchecked() - c.elements_width()) / 2.0) as i32,
                        ),
                        0.0,
                    ),
                    Alignment::Right => (
                        top_left
                            .move_x_by((c.width.get_fixed_unchecked() - c.elements_width()) as i32),
                        0.0,
                    ),
                    Alignment::Justified => (
                        top_left,
                        (c.width.get_fixed_unchecked() - c.elements_width())
                            / (c.elements.len() - 1) as f32,
                    ),
                };

                let mut top_left = top_left;
                let mut depth = top_left.y;

                for element in c.elements.iter() {
                    depth =
                        element.compute_textbox_positions(textbox_positions, top_left, font_dict);
                    top_left = top_left.move_x_by(
                        element.width().get_fixed_unchecked() as i32 + per_elem_space as i32,
                    );
                }
                depth
            }
            Layout::FrozenRow(_) => {
                unreachable!("Cannot compute textbox positions of frozen row: {:?}", self)
            }
            Layout::Text(e) => {
                let width = e.width.get_fixed_unchecked();
                let height = e.font.get_height(font_dict);
                let textbox = SpatialBox::new(
                    top_left.clone(),
                    top_left.move_x_by(width as i32).move_y_by(height as i32),
                );
                textbox_positions.push((textbox, e.clone()));

                top_left.y + height as u32
            }
            Layout::Ref(_) => {
                todo!("Should not be able to compute textbox positions of uninstantiated layout")
            }
        }
    }
}

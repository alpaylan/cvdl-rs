use std::collections::HashMap;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::alignment::Alignment;
use crate::basic_layout::BasicLayout;
use crate::container::Container;
use crate::document::DocumentDefinition;
use crate::element::Element;

use crate::font::{Font, FontDict};
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

impl Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layout::Stack(container) => write!(f, "{}", container),
            Layout::FrozenRow(container) => write!(f, "{}", container),
            Layout::FlexRow(container) => write!(f, "{}", container),
            Layout::Text(element) => write!(f, "{}", element),
            Layout::Ref(element) => write!(f, "{}", element),
        }
    }
}

impl Layout {
    pub fn new_stack(container: Container) -> Layout {
        log::debug!("Creating new stack: {}", container.uid);
        Layout::Stack(container)
    }

    pub fn new_frozen_row(container: Container) -> Layout {
        log::debug!("Creating new frozen row: {}", container.uid);
        Layout::FrozenRow(container)
    }

    pub fn new_flex_row(container: Container) -> Layout {
        log::debug!("Creating new flex row: {}", container.uid);
        Layout::FlexRow(container)
    }

    pub fn new_text(element: Element) -> Layout {
        log::debug!("Creating new text element: {}", element.uid);
        Layout::Text(element)
    }

    pub fn new_ref(element: Element) -> Layout {
        log::debug!("Creating new ref element: {}", element.uid);
        Layout::Ref(element)
    }
}

impl Layout {
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn margin(&self) -> Margin {
        match self {
            Layout::Stack(container)
            | Layout::FrozenRow(container)
            | Layout::FlexRow(container) => container.margin,
            Layout::Text(element) | Layout::Ref(element) => element.margin,
        }
    }
    #[allow(dead_code)]
    pub fn alignment(&self) -> Alignment {
        match self {
            Layout::Stack(container)
            | Layout::FrozenRow(container)
            | Layout::FlexRow(container) => container.alignment,
            Layout::Text(element) | Layout::Ref(element) => element.alignment,
        }
    }

    pub fn fonts(&self) -> Vec<Font> {
        match self {
            Layout::Stack(container)
            | Layout::FrozenRow(container)
            | Layout::FlexRow(container) => container.fonts(),
            Layout::Text(element) | Layout::Ref(element) => vec![element.font.clone()],
        }
    }
    #[allow(dead_code)]
    pub fn with_margin(&self, margin: Margin) -> Layout {
        match self {
            Layout::Stack(container) => Layout::new_stack(container.with_margin(margin)),
            Layout::FrozenRow(container) => Layout::new_frozen_row(container.with_margin(margin)),
            Layout::FlexRow(container) => Layout::new_flex_row(container.with_margin(margin)),
            Layout::Text(element) => Layout::new_text(element.with_margin(margin)),
            Layout::Ref(element) => Layout::new_ref(element.with_margin(margin)),
        }
    }
    #[allow(dead_code)]
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
        log::debug!("Checking if {} is instantiated...", self);
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
            let mut element = element.with_item(text.to_string());

            if let ItemContent::Url { url, text: _ } = text {
                element = element.with_url(url.clone())
            }

            Layout::Text(element)
        } else {
            Layout::Stack(Container::empty_container())
        }
    }

    pub fn bound_width(&self, width: f32) -> Layout {
        let bound = match self.width() {
            Width::Absolute(w) => {
                f32::min(w, width)
            },
            Width::Percentage(_) => unreachable!("Layout::bound_width: Cannot bounded width for non-unitized widths!"),
            Width::Fill => width,
        };

        match self {
            Layout::Stack(c) => Layout::new_stack(c.bound_width(bound)),
            Layout::FrozenRow(c) => Layout::new_frozen_row(c.bound_width(bound)),
            Layout::FlexRow(c) => Layout::new_flex_row(c.bound_width(bound)),
            Layout::Text(e) => Layout::new_text(e.bound_width(bound)),
            Layout::Ref(_) => unreachable!("Cannot propagate widths of uninstantiated layout"),
        }
    }

    pub fn scale_width(&self, document_width: f32) -> Layout {
        match self {
            Layout::Stack(c) => Layout::new_stack(c.scale_width(document_width)),
            Layout::FrozenRow(c) => Layout::new_frozen_row(c.scale_width(document_width)),
            Layout::FlexRow(c) => Layout::new_flex_row(c.scale_width(document_width)),
            Layout::Text(e) => Layout::new_text(e.scale_width(document_width)),
            Layout::Ref(_) => unreachable!("Cannot scale width of uninstantiated layout"),
        }
    }

    pub fn normalize(&self, document: &DocumentDefinition, font_dict: &FontDict) -> Layout {
        log::debug!("Normalizing document, checking if {} is instantiated...", self);

        if !self.is_instantiated() {
            log::error!("Document is not instantiated {}", self);
            panic!("Cannot normalize uninstantiated layout");
        };

        log::debug!("Document is instantiated. Scaling widths...");

        let scaled_layout = self.scale_width(document.width);

        log::debug!("Widths are scaled. Bounding widths...");

        let bounded_layout = scaled_layout.bound_width(document.width);

        log::debug!("Widths are bounded. Filling fonts...");

        let font_filled_layout = bounded_layout.fill_fonts(font_dict);

        log::debug!("Fonts filled. Breaking lines...");

        let broken_layout = font_filled_layout.break_lines(font_dict);

        log::debug!("Lines broken.");

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
                        uid: c.uid,
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
        height_offset: f32,
        font_dict: &FontDict,
    ) -> (f32, Vec<(SpatialBox, Element)>) {
        let mut textbox_positions: Vec<(SpatialBox, Element)> = Vec::new();
        let top_left: Point = Point::new(0.0, height_offset);
        let depth = self.compute_textbox_positions(&mut textbox_positions, top_left, font_dict);

        (depth, textbox_positions)
    }

    fn compute_textbox_positions(
        &self,
        textbox_positions: &mut Vec<(SpatialBox, Element)>,
        top_left: Point,
        font_dict: &FontDict,
    ) -> f32 {
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
                            (c.width.get_fixed_unchecked() - c.elements_width()) / 2.0,
                        ),
                        0.0,
                    ),
                    Alignment::Right => (
                        top_left
                            .move_x_by(c.width.get_fixed_unchecked() - c.elements_width()),
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
                        element.width().get_fixed_unchecked() + per_elem_space,
                    );
                }
                depth
            }
            Layout::FrozenRow(_) => {
                unreachable!("Cannot compute textbox positions of frozen row: {:?}", self)
            }
            Layout::Text(e) => {
                let width = e.text_width.get_fixed_unchecked();
                let height = e.font.get_height(font_dict);
                let textbox = SpatialBox::new(
                    top_left.clone(),
                    top_left.move_x_by(width).move_y_by(height),
                );
                textbox_positions.push((textbox, e.clone()));

                top_left.y + height
            }
            Layout::Ref(_) => {
                todo!("Should not be able to compute textbox positions of uninstantiated layout")
            }
        }
    }
}

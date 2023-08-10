use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    alignment::Alignment, font::FontDict, layout::Layout, margin::Margin, resume_data::ItemContent,
    width::Width,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Container {
    pub elements: Vec<Layout>,
    #[serde(default = "Margin::default")]
    pub margin: Margin,
    #[serde(default = "Alignment::default")]
    pub alignment: Alignment,
    #[serde(default = "Width::default")]
    pub width: Width,
}

impl Container {
    pub fn empty_container() -> Container {
        Container {
            elements: vec![],
            margin: Margin::default(),
            alignment: Alignment::default(),
            width: Width::default(),
        }
    }

    pub fn with_elements(&self, elements: Vec<Layout>) -> Container {
        Container {
            elements,
            margin: self.margin,
            alignment: self.alignment,
            width: self.width,
        }
    }
    pub fn with_margin(&self, margin: Margin) -> Container {
        Container {
            elements: self.elements.clone(),
            margin,
            alignment: self.alignment,
            width: self.width,
        }
    }

    pub fn with_alignment(&self, alignment: Alignment) -> Container {
        Container {
            elements: self.elements.clone(),
            margin: self.margin,
            alignment,
            width: self.width,
        }
    }

    pub fn with_width(&self, width: Width) -> Container {
        Container {
            elements: self.elements.clone(),
            margin: self.margin,
            alignment: self.alignment,
            width,
        }
    }

    pub fn instantiate(&self, section: &HashMap<String, ItemContent>) -> Container {
        Container {
            elements: self
                .elements
                .iter()
                .map(|e| e.instantiate(section))
                .collect(),
            margin: self.margin,
            alignment: self.alignment,
            width: self.width,
        }
    }

    pub fn bound_width(&self, width: f32) -> Container {
        let bound = if self.width.is_fixed() && self.width.get_fixed().unwrap() <= width {
            self.width.get_fixed().unwrap()
        } else {
            width
        };

        Container {
            elements: self.elements.iter().map(|e| e.bound_width(bound)).collect(),
            margin: self.margin,
            alignment: self.alignment,
            width: Width::Fixed(bound),
        }
    }

    pub fn scale_width(&self, w: u32) -> Container {
        Container {
            elements: self.elements.iter().map(|e| e.scale_width(w)).collect(),
            margin: self.margin,
            alignment: self.alignment,
            width: self.width.scale(w as f32 / 100.0),
        }
    }

    pub fn fill_fonts(&self, fonts: &FontDict) -> Container {
        Container {
            elements: self.elements.iter().map(|e| e.fill_fonts(fonts)).collect(),
            margin: self.margin,
            alignment: self.alignment,
            width: self.width,
        }
    }

    pub fn break_lines(&self, font_dict: &FontDict) -> Vec<Container> {
        
        let mut lines: Vec<Container> = vec![];
        let mut current_line: Vec<Layout> = vec![];
        let mut current_width = 0.0;
        let elements: Vec<Layout> = self.elements.iter().map(|e| e.break_lines(font_dict)).collect();

        for element in elements {
            let element_width = element.width().get_fixed_unchecked();
            if current_width + element_width > self.width.get_fixed().unwrap() {
                lines.push(self.with_elements(current_line));
                current_line = vec![];
                current_width = 0.0;
            }
            current_line.push(element.clone());
            current_width += element_width;
        }

        if !current_line.is_empty() {
            lines.push(self.with_elements(current_line));
        }

        lines
    }

    pub fn elements_width(&self) -> f32 {
        self.elements
            .iter()
            .map(|e| e.width().get_fixed_unchecked())
            .sum()
    }
}

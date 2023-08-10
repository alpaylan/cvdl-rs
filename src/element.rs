use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{alignment::Alignment, font::Font, margin::Margin, width::Width};
use rusttype::Font as RFont;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Element {
    pub item: String,
    #[serde(default = "Margin::default")]
    pub margin: Margin,
    #[serde(default = "Alignment::default")]
    pub alignment: Alignment,
    #[serde(default = "Width::default")]
    pub width: Width,
    #[serde(default = "Width::default")]
    pub text_width: Width,
    #[serde(default = "Font::default")]
    pub font: Font,
    #[serde(skip)]
    #[serde(default = "bool::default")]
    pub is_fill: bool,
}

impl Default for Element {
    fn default() -> Element {
        Element {
            item: String::new(),
            margin: Margin::default(),
            alignment: Alignment::default(),
            width: Width::default(),
            text_width: Width::default(),
            font: Font::default(),
            is_fill: false,
        }
    }
}

impl Element {
    pub fn with_item(&self, item: String) -> Element {
        Element {
            item,
            margin: self.margin,
            alignment: self.alignment,
            width: self.width,
            text_width: self.text_width,
            font: self.font.clone(),
            is_fill: self.is_fill,
        }
    }

    pub fn with_margin(&self, margin: Margin) -> Element {
        Element {
            item: self.item.clone(),
            margin,
            alignment: self.alignment,
            width: self.width,
            text_width: self.text_width,
            font: self.font.clone(),
            is_fill: self.is_fill,
        }
    }

    pub fn with_alignment(&self, alignment: Alignment) -> Element {
        Element {
            item: self.item.clone(),
            margin: self.margin,
            alignment,
            width: self.width,
            text_width: self.text_width,
            font: self.font.clone(),
            is_fill: self.is_fill,
        }
    }

    pub fn with_width(&self, width: Width) -> Element {
        Element {
            item: self.item.clone(),
            margin: self.margin,
            alignment: self.alignment,
            width,
            text_width: self.text_width,
            font: self.font.clone(),
            is_fill: self.is_fill,
        }
    }

    pub fn with_text_width(&self, text_width: Width) -> Element {
        Element {
            item: self.item.clone(),
            margin: self.margin,
            alignment: self.alignment,
            width: self.width,
            text_width,
            font: self.font.clone(),
            is_fill: self.is_fill,
        }
    }

    pub fn scale_width(&self, w: u32) -> Element {
        Element {
            item: self.item.clone(),
            margin: self.margin,
            alignment: self.alignment,
            width: self.width.scale(w as f32 / 100.0),
            text_width: self.text_width,
            font: self.font.clone(),
            is_fill: self.is_fill,
        }
    }

    pub fn fill_fonts(&self, fonts: &HashMap<String, RFont>) -> Element {
        let text_width_with_font = self.font.get_width(&self.item, fonts);
        if self.is_fill {
            Element {
                item: self.item.clone(),
                margin: self.margin,
                alignment: self.alignment,
                width: Width::Fixed(f32::min(
                    self.width.get_fixed().unwrap(),
                    text_width_with_font,
                )),
                text_width: Width::Fixed(text_width_with_font),
                font: self.font.clone(),
                is_fill: self.is_fill,
            }
        } else {
            Element {
                item: self.item.clone(),
                margin: self.margin,
                alignment: self.alignment,
                width: self.width,
                text_width: Width::Fixed(text_width_with_font),
                font: self.font.clone(),
                is_fill: self.is_fill,
            }
        }
        
    }

    pub fn break_lines(&self, font_dict: &HashMap<String, RFont>) -> Vec<Element> {
        if self.text_width.get_fixed_unchecked() <= self.width.get_fixed_unchecked() {
            return vec![self.clone()];
        }

        let mut lines: Vec<Element> = vec![];

        // todo: I'm sure this implementation is pretty buggy. Note to future me, fix
        // this.
        let words = self.item.split_whitespace().collect::<Vec<&str>>();
        let mut line = String::new();
        for word in words {
            let candidate_line = line.clone() + " " + word;
            let candidate_width: f32 = self.font.get_width(&candidate_line, font_dict);

            if candidate_width > self.width.get_fixed_unchecked() {
                line.pop();
                lines.push(self.with_item(line));
                line = String::new();
            }

            line.push_str(word);
            line.push(' ');
        }

        line.pop();
        if line.len() > 0 {
            lines.push(self.with_item(line));
        }

        lines
    }

    pub fn bound_width(&self, width: f32) -> Element {
        if self.width.is_fixed() {
            Element {
                item: self.item.clone(),
                margin: self.margin,
                alignment: self.alignment,
                width: Width::Fixed(f32::min(self.width.get_fixed_unchecked(), width)),
                text_width: self.text_width,
                font: self.font.clone(),
                is_fill: false,
            }
        } else {
            Element {
                item: self.item.clone(),
                margin: self.margin,
                alignment: self.alignment,
                width: Width::Fixed(width),
                text_width: self.text_width,
                font: self.font.clone(),
                is_fill: true,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_break_lines() {
        let mut font_dict = HashMap::new();
        let font_data = include_bytes!("../assets/Exo/static/Exo-Medium.ttf");
        // This only succeeds if collection consists of one font
        let _font = RFont::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");
        font_dict.insert("Arial".to_string(), _font);

        let element = Element {
            item: "hello world".to_string(),
            margin: Margin::default(),
            alignment: Alignment::default(),
            width: Width::Fixed(100.0),
            text_width: Width::default(),
            font: Font::default(),
            is_fill: false,
        };

        let element = element.fill_fonts(&font_dict);

        let lines = element.break_lines(&font_dict);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].item, "hello world");

        let element = Element {
            item: "hello world".to_string(),
            margin: Margin::default(),
            alignment: Alignment::default(),
            width: Width::Fixed(22.0),
            text_width: Width::default(),
            font: Font::default(),
            is_fill: false,
        };

        let element = element.fill_fonts(&font_dict);

        let lines = element.break_lines(&font_dict);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].item, "hello");
        assert_eq!(lines[1].item, "world");
    }
}

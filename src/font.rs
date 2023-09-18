use std::{collections::HashMap, fs::File, io::Read};

use font_kit::properties::{Style, Weight};
use printpdf::IndirectFontRef;
use rusttype::{point, Scale};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Font {
    #[serde(default = "Font::default_name")]
    pub name: String,
    #[serde(default = "Font::default_size")]
    pub size: f32,
    #[serde(default = "FontWeight::default")]
    pub weight: FontWeight,
    #[serde(default = "FontStyle::default")]
    pub style: FontStyle,
    #[serde(default = "FontSource::default")]
    pub source: FontSource,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FontSource {
    Local,
    System,
}

impl Default for FontSource {
    fn default() -> Self {
        FontSource::System
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FontWeight {
    Light,
    Medium,
    Bold,
}

impl Default for FontWeight {
    fn default() -> Self {
        FontWeight::Medium
    }
}

impl ToString for FontWeight {
    fn to_string(&self) -> String {
        match self {
            FontWeight::Light => "Light".to_string(),
            FontWeight::Medium => "Medium".to_string(),
            FontWeight::Bold => "Bold".to_string(),
        }
    }
}

impl Into<Weight> for FontWeight {
    fn into(self) -> Weight {
        match self {
            FontWeight::Light => Weight::LIGHT,
            FontWeight::Medium => Weight::MEDIUM,
            FontWeight::Bold => Weight::BOLD,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FontStyle {
    Normal,
    Italic,
}

impl Default for FontStyle {
    fn default() -> Self {
        FontStyle::Normal
    }
}

impl ToString for FontStyle {
    fn to_string(&self) -> String {
        match self {
            FontStyle::Normal => "".to_string(),
            FontStyle::Italic => "Italic".to_string(),
        }
    }
}

impl Into<Style> for FontStyle {
    fn into(self) -> Style {
        match self {
            FontStyle::Normal => Style::Normal,
            FontStyle::Italic => Style::Italic,
        }
    }
}

impl Default for Font {
    fn default() -> Font {
        Font {
            name: Font::default_name(),
            size: Font::default_size(),
            weight: FontWeight::default(),
            style: FontStyle::default(),
            source: FontSource::default(),
        }
    }
}

impl Font {
    pub fn default_name() -> String {
        "Arial".to_string()
    }

    pub fn default_size() -> f32 {
        12.0
    }
}

pub struct LoadedFont {
    pub printpdf_font: IndirectFontRef,
    pub rusttype_font: rusttype::Font<'static>,
}

pub type FontDict = HashMap<String, LoadedFont>;

pub trait FontLoader {
    fn load_from_path(&mut self, doc: &printpdf::PdfDocumentReference, name: String, path: String);
}

impl FontLoader for FontDict {
    fn load_from_path(&mut self, doc: &printpdf::PdfDocumentReference, name: String, path: String) {
        let mut file = File::open(path).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();

        let rusttype_font = rusttype::Font::try_from_vec(bytes.clone()).unwrap();
        let printpdf_font = doc.add_external_font(bytes.as_slice()).unwrap();
        self.insert(
            name,
            LoadedFont {
                printpdf_font,
                rusttype_font,
            },
        );
    }
}

impl Font {
    pub fn full_name(&self) -> String {
        self.name.clone() + "-" + self.weight.to_string().as_str() + self.style.to_string().as_str()
    }

    pub fn get_width(&self, text: &String, font_dict: &FontDict) -> f32 {
        // The font size to use
        let scale = Scale::uniform(self.size);
        let font = &font_dict
            .get(&self.full_name())
            .unwrap_or_else(|| font_dict.get(&Font::default().full_name()).unwrap())
            .rusttype_font;

        // The text to render
        let v_metrics = font.v_metrics(scale);

        // layout the glyphs in a line with 20 pixels padding
        let glyphs: Vec<_> = font
            .layout(text.trim(), scale, point(0_f32, v_metrics.ascent))
            .collect();

        let glyphs_width = {
            let min_x = glyphs
                .first()
                .map(|g| g.pixel_bounding_box().unwrap().min.x)
                .unwrap();
            let max_x = glyphs
                .last()
                .map(|g| g.pixel_bounding_box().unwrap().max.x)
                .unwrap();
            (max_x - min_x) as f32
        };

        glyphs_width
    }

    pub fn get_height(&self, font_dict: &FontDict) -> f32 {
        // The font size to use
        let scale = Scale::uniform(self.size);
        let font = &font_dict
            .get(&self.full_name())
            .unwrap_or_else(|| &font_dict.get(&Font::default().full_name()).unwrap())
            .rusttype_font;

        // The text to render
        let v_metrics = font.v_metrics(scale);

        // work out the layout size
        let glyphs_height = v_metrics.ascent - v_metrics.descent;

        glyphs_height
    }
}

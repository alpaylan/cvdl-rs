use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use rusttype::{point, Font as RFont, Scale};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Font {
    #[serde(default = "Font::default_name")]
    pub name: String,
    #[serde(default = "Font::default_size")]
    pub size: f32,
    #[serde(default = "FontWeight::default")]
    pub weight: FontWeight,
    #[serde(default = "FontSlope::default")]
    pub slope: FontSlope,
    #[serde(default = "FontSource::default")]
    pub source: FontSource
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FontSource {
    Local,
    System
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FontSlope {
    Normal,
    Italic,
}

impl Default for FontSlope {
    fn default() -> Self {
        FontSlope::Normal
    }
}

impl ToString for FontSlope {
    fn to_string(&self) -> String {
        match self {
            FontSlope::Normal => "".to_string(),
            FontSlope::Italic => "Italic".to_string(),
        }
    }
}

impl Default for Font {
    fn default() -> Font {
        Font {
            name: Font::default_name(),
            size: Font::default_size(),
            weight: FontWeight::default(),
            slope: FontSlope::default(),
            source: FontSource::default()
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

pub type FontDict<'a> = HashMap<String, RFont<'a>>;

impl Font {
    pub fn full_name(&self) -> String {
        self.name.clone() + "-" + self.weight.to_string().as_str() + self.slope.to_string().as_str()
    }

    pub fn get_width(&self, text: &String, font_dict: &FontDict) -> f32 {
        // The font size to use
        let scale = Scale::uniform(self.size);
        let font = font_dict.get(&self.name).unwrap();
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

    pub fn get_height(&self, font_dict: &HashMap<String, RFont>) -> f32 {
        // The font size to use
        let scale = Scale::uniform(self.size);
        let font = font_dict.get(&self.name).unwrap();

        // The text to render
        let v_metrics = font.v_metrics(scale);

        // work out the layout size
        let glyphs_height = v_metrics.ascent - v_metrics.descent;

        glyphs_height
    }
}

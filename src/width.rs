use serde::{Deserialize, Serialize, de::Visitor};

use crate::width;


#[derive(Debug, Clone, Copy)]
pub enum Width {
    Absolute(f32),
    Percentage(f32),
    Fill,
}

impl<'de> Visitor<'de> for Width {
    type Value = Width;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string representing a width")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        if v.ends_with('%') {
            let w = v[..v.len() - 1].parse::<f32>().map_err(|_| E::custom("invalid percentage"))?;
            Ok(Width::Percentage(w))
        } else if v.ends_with("px") {
            let w = v[..v.len() - 2].parse::<f32>().map_err(|_| E::custom("invalid pixel width"))?;
            Ok(Width::Absolute(w))
        } else if v == "fill" {
            Ok(Width::Fill)
        } else {
            Err(E::custom("invalid width"))
        }
    }
}

impl<'de> Deserialize<'de> for Width {
    fn deserialize<D>(deserializer: D) -> Result<Width, D::Error>
        where
            D: serde::Deserializer<'de>, {
        deserializer.deserialize_str(Width::Fill)
    }
}

impl Serialize for Width {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer, {
        match self {
            Width::Absolute(w) => serializer.serialize_str(&format!("{}px", w)),
            Width::Percentage(w) => serializer.serialize_str(&format!("{}%", w)),
            Width::Fill => serializer.serialize_str("fill"),
        }
    }
}


impl Default for Width {
    fn default() -> Self {
        return Width::Fill;
    }
}

impl Width {
    pub fn is_fixed(&self) -> bool {
        match self {
            Width::Fill => false,
            _ => true,
        }
    }

    pub fn get_fixed(&self) -> Option<f32> {
        match self {
            Width::Absolute(w) | Width::Percentage(w) => Some(*w),
            Width::Fill => None,
        }
    }

    pub fn get_fixed_unchecked(&self) -> f32 {
        match self {
            Width::Absolute(w) | Width::Percentage(w) => *w,
            Width::Fill => panic!("Width::get_fixed_unchecked() called on Width::Fill"),
        }
    }


    pub fn scale(&self, total_width: f32) -> Width {
        match self {
            Width::Percentage(w) => Width::Absolute(*w / 100.0 * total_width),
            Width::Absolute(w) => *self,
            Width::Fill => Width::Fill,
        }
    }
}

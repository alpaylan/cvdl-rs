use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(untagged)]
pub enum Width {
    Fixed(f32),
    Fill,
}

impl Default for Width {
    fn default() -> Self {
        return Width::Fill;
    }
}

impl Width {
    pub fn is_fixed(&self) -> bool {
        match self {
            Width::Fixed(_) => true,
            Width::Fill => false,
        }
    }

    pub fn get_fixed(&self) -> Option<f32> {
        match self {
            Width::Fixed(w) => Some(*w),
            Width::Fill => None,
        }
    }

    pub fn get_fixed_unchecked(&self) -> f32 {
        match self {
            Width::Fixed(w) => *w,
            Width::Fill => panic!("Width::get_fixed_unchecked() called on Width::Fill"),
        }
    }


    pub fn scale(&self, scale: f32) -> Width {
        match self {
            Width::Fixed(w) => Width::Fixed(*w * scale),
            Width::Fill => Width::Fill,
        }
    }
}

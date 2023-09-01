use std::collections::HashMap;
use rusttype::{point, Font, Scale};

pub struct DocumentDefinition {
    pub font_dict: HashMap<String, Font<'static>>,
    pub width: f32,
    pub height: f32,
}
mod alignment;
mod basic_layout;
mod container;
mod data_schema;
mod document;
mod element;
mod font;
mod layout;
mod layout_schema;
mod margin;
mod pdf_layout;
mod point;
mod resume_data;
mod spatial_box;
mod width;

use std::fs;

use document::DocumentDefinition;
use font::FontDict;
use pdf_layout::PdfLayout;
use rusttype::Font as RFont;
use std::path::Path;

use std::env;

fn main() {
    env_logger::init();

    let data_schema_path = env::args().nth(1).expect("No data schema path provided");
    let schema = fs::read_to_string(data_schema_path).unwrap();
    let data_schemas = data_schema::DataSchema::from_json(&schema);

    let layout_schema_path = env::args().nth(2).expect("No layout schema path provided");
    let schema = fs::read_to_string(layout_schema_path).unwrap();
    let layout_schemas = layout_schema::LayoutSchema::from_json(&schema);

    let resume_path = env::args().nth(3).expect("No resume path provided");
    let resume = fs::read_to_string(resume_path).unwrap();
    let resume_data = resume_data::ResumeData::from_json(&resume);

    let mut font_dict = FontDict::new();

    // This only succeeds if collection consists of one font
    let _font =
        RFont::try_from_bytes(include_bytes!("../assets/Exo/static/Exo-Medium.ttf") as &[u8])
            .expect("Error constructing Font");

    font_dict.insert("Exo".to_string(), _font);

    let _font = RFont::try_from_bytes(include_bytes!("../assets/Exo/static/Exo-Bold.ttf") as &[u8])
        .expect("Error constructing Font");

    font_dict.insert("Exo-Bold".to_string(), _font);

    let pdf_layout = PdfLayout {
        doc: DocumentDefinition {
            width: 612,
            height: 792,
            font_dict,
        },
    };

    let results_path = env::args().nth(4).expect("No results path provided");
    let debug = if let Some(is_debug) = env::args().nth(5) {
        is_debug == "--debug"
    } else {
        false
    };

    pdf_layout
        .render(
            layout_schemas,
            resume_data,
            data_schemas,
            Path::new(results_path.as_str()),
            debug
        )
        .unwrap();
}

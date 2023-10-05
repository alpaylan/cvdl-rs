#![feature(file_create_new)]

mod alignment;
mod any_layout;
mod basic_layout;
mod container;
mod data_schema;
mod element;
mod font;
mod layout;
mod layout_schema;
mod local_storage;
mod margin;
mod pdf_layout;
mod png_layout;
mod point;
mod resume_data;
mod resume_layout;
mod spatial_box;
mod width;

use std::fs;

use pdf_layout::PdfLayout;
use png_layout::PngLayout;
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

    let resume_layout_path = env::args().nth(3).expect("No resume layout path provided");
    let schema = fs::read_to_string(resume_layout_path).unwrap();
    let resume_layouts = resume_layout::ResumeLayout::from_json(&schema);

    let resume_path = env::args().nth(4).expect("No resume path provided");
    let resume = fs::read_to_string(resume_path).unwrap();
    let resume_data = resume_data::ResumeData::from_json(&resume);

    let resume_layout = resume_layouts
        .iter()
        .find(|layout| layout.schema_name == resume_data.layout)
        .unwrap();

    let results_path = env::args().nth(5).expect("No results path provided");
    let debug = if let Some(is_debug) = env::args().nth(6) {
        is_debug == "--debug"
    } else {
        false
    };

    PdfLayout::render(
        layout_schemas,
        resume_data,
        data_schemas,
        resume_layout.clone(),
        Path::new(results_path.as_str()),
        debug,
    )
    .unwrap();
}

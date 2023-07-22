mod data_schema;
mod resume_data;
mod text_layout_schema;

use std::{fs, path::Path};

fn main() {
    let schema = fs::read_to_string("data/data-schemas.json").unwrap();
    let data_schemas = data_schema::DataSchema::from_json(&schema);

    let resume = fs::read_to_string("data/resume.json").unwrap();
    let resume_data = resume_data::ResumeData::from_json(&resume);

    let text_layout = fs::read_to_string("data/text-layout-schemas.json").unwrap();
    let text_layout_schemas = text_layout_schema::TextLayoutSchema::from_json(&text_layout);

    let output = text_layout_schema::TextLayoutSchema::render(text_layout_schemas, resume_data, data_schemas,  Path::new("output.md"));
    
    println!("{:?}", output);

}

mod data_schema;
mod resume_data;
mod text_layout_schema;

use std::fs;

fn main() {
    let schema = fs::read_to_string("data/data-schemas.json").unwrap();
    let data_schemas = data_schema::DataSchema::from_json(&schema);

    let resume = fs::read_to_string("data/resume.json").unwrap();
    let resume_data = resume_data::ResumeData::from_json(&resume);

    let text_layout = fs::read_to_string("data/text-layout-schemas.json").unwrap();
    let text_layout_schemas = text_layout_schema::TextLayoutSchema::from_json(&text_layout);

    for section in resume_data.sections {
        println!("# {}", section.name);
        let schema = data_schemas.iter().find(|s| s.name == section.schema).unwrap();
        let text_layout_schema = text_layout_schemas.iter().find(|s| s.schema_name == section.schema).unwrap();
        let mut blueprint = text_layout_schema.text_layout_schema.clone();
        for field in &schema.fields {
            let resume_field = section.elements.iter().find(|e| e.contains_key(&field.name)).unwrap();
            blueprint = blueprint.replace(&format!("{{{}}}", field.name), resume_field.get(&field.name).unwrap());
        }
        println!("{}\n", blueprint);
    }


}

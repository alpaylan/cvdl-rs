mod data_schema;
mod resume_data;
mod text_layout_schema;
mod ascii_layout_schema;
mod point;
mod spatial_box;

use ascii_layout_schema::{Layout, LayoutGrid, GridElement, Element, Font, Alignment, Margin};

use std::{fs, path::Path};

use crate::ascii_layout_schema::LayoutSchema;

fn main() {
    let schema = fs::read_to_string("data/data-schemas.json").unwrap();
    let data_schemas = data_schema::DataSchema::from_json(&schema);

    let resume = fs::read_to_string("data/resume2.json").unwrap();
    let resume_data = resume_data::ResumeData::from_json(&resume);

    // let text_layout = fs::read_to_string("data/text-layout-schemas.json").unwrap();
    // let text_layout_schemas = text_layout_schema::TextLayoutSchema::from_json(&text_layout);

    // let output = text_layout_schema::TextLayoutSchema::render(text_layout_schemas, resume_data, data_schemas,  Path::new("output.md"));
    
    // println!("{:?}", output);


    let header_layout = Layout::Stack(vec![
        Layout::Element(Element {
            text: String::from("Title"),
            font: Font {
                name: String::from("Arial"),
                size: 12,
            },
            alignment: Alignment::Left,
        }),
    ]);

    // "Company": "Emproof",
    // "Position": "Embedded Security Engineering Intern",
    // "Date-Started": "Jun 2019",
    // "Date-Finished": "Sep 2019",
    // "Text": "Worked on translation validation of binary obfuscation techniques.",
    // "Skills": ["C++", "Z3 SMT Solver", "ARM Assembly", "Symbolic Execution"]

    let layout = Layout::Stack(vec![
        Layout::just_grid(vec![
            Layout::elem("Company"),
            Layout::just_grid(vec![
                Layout::elem("Date-Started"),
                Layout::elem("-"),
                Layout::elem("Date-Finished"),
            ])
        ]),
        Layout::just_grid(vec![
            Layout::elem("Position")
        ]),
        Layout::just_grid(vec![
            Layout::elem("Text")
        ]),
        Layout::just_grid(vec![
            Layout::elem("Skills")
        ])
    ]);

    let layout_schema = LayoutSchema {
        schema_name: "Work-Experience".to_string(),
        header_layout_schema: header_layout,
        item_layout_schema: layout,
    };

    print!("{:?}", Layout::render(
        vec![layout_schema],
        resume_data,
        data_schemas,
        Path::new("output.txt")
    ));

}


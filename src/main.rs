mod data_schema;
mod resume_data;
mod text_layout_schema;
mod layout_schema;

use layout_schema::{Layout, LayoutGrid, GridElement, Element, Font, Alignment, Margin};

use std::{fs, path::Path};

fn main() {
    // let schema = fs::read_to_string("data/data-schemas.json").unwrap();
    // let data_schemas = data_schema::DataSchema::from_json(&schema);

    // let resume = fs::read_to_string("data/resume.json").unwrap();
    // let resume_data = resume_data::ResumeData::from_json(&resume);

    // let text_layout = fs::read_to_string("data/text-layout-schemas.json").unwrap();
    // let text_layout_schemas = text_layout_schema::TextLayoutSchema::from_json(&text_layout);

    // let output = text_layout_schema::TextLayoutSchema::render(text_layout_schemas, resume_data, data_schemas,  Path::new("output.md"));
    
    // println!("{:?}", output);

    let layout = Layout::Stack(vec![Layout::Grid(LayoutGrid {
        elements: vec![
            GridElement {
                min_width: None,
                max_width: None,
                default_width: None,
                margin: Margin::default(),
                element: Layout::Element(Element {
                    text: String::from("Position"),
                    font: Font {
                        name: String::from("Arial"),
                        size: 12,
                    },
                    alignment: Alignment::Left,
                }),
            },
            GridElement {
                min_width: None,
                max_width: None,
                default_width: None,
                margin: Margin::default(),
                element: Layout::Element(Element {
                    text: String::from("Date"),
                    font: Font {
                        name: String::from("Arial"),
                        size: 12,
                    },
                    alignment: Alignment::Left,
                }),
            },
        ],
        alignment: Alignment::Justified,
    }),
    Layout::Grid(LayoutGrid {
        elements: vec![
            GridElement {
                min_width: None,
                max_width: None,
                default_width: None,
                margin: Margin::default(),
                element: Layout::Element(Element {
                    text: String::from("Company"),
                    font: Font {
                        name: String::from("Arial"),
                        size: 12,
                    },
                    alignment: Alignment::Left,
                }),
            },
            GridElement {
                min_width: None,
                max_width: None,
                default_width: None,
                margin: Margin::default(),
                element: Layout::Grid(LayoutGrid { elements: vec![
                    GridElement { min_width: None, max_width: None, default_width: None, margin: Margin { top: 0, bottom: 0, left: 0, right: 2 }, element: Layout::Element(Element { text: String::from("Date Started"), font: Font { name: String::from("Arial"), size: 12 }, alignment: Alignment::Left }) }
                    , GridElement { min_width: None, max_width: None, default_width: None, margin: Margin::default(), element: Layout::Element(Element { text: String::from("Date Finished"), font: Font { name: String::from("Arial"), size: 12 }, alignment: Alignment::Left }) }
                ], alignment: Alignment::Left }),
            },
        ],
        alignment: Alignment::Justified,
    })
    ]);

    print!("{}", layout.blueprint());

}


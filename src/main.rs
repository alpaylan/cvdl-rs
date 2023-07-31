mod ascii_layout_schema;
mod data_schema;
mod point;
mod resume_data;
mod spatial_box;
mod text_layout_schema;

use ascii_layout_schema::{Alignment, Element, Layout, Margin};

use std::{fs, path::Path};

use crate::ascii_layout_schema::{Container, ContainerInner, LayoutSchema, Stack, Width};

fn main() {
    let schema = fs::read_to_string("data/data-schemas.json").unwrap();
    let data_schemas = data_schema::DataSchema::from_json(&schema);

    let resume = fs::read_to_string("data/resume2.json").unwrap();
    let resume_data = resume_data::ResumeData::from_json(&resume);

    let layout_schemas = vec![LayoutSchema {
        schema_name: "Work-Experience".to_string(),
        header_layout_schema: Layout::Container(Container {
            inner: ContainerInner::Element(Element::Ref("Title".to_string())),
            width: Width::Fixed(100),
            alignment: Alignment::Left,
            margin: Margin::new(0, 0, 0, 0),
        }),
        item_layout_schema: Layout::mk_stack(vec![
            Layout::mk_row(vec![
                Layout::mk_ref("Company".to_string()).with_width(70),
                Layout::mk_row(vec![
                    Layout::mk_ref("Date-Started".to_string()),
                    Layout::mk_text("-".to_string()).with_margin(Margin::new(0, 0, 1, 1)),
                    Layout::mk_ref("Date-Finished".to_string()),
                ])
                .with_width(30)
                .with_alignment(Alignment::Right),
            ]),
            Layout::mk_ref("Position".to_string()).with_width(70),
            Layout::mk_ref("Text".to_string()).with_width(70),
            Layout::mk_ref("Skills".to_string()).with_width(70),
        ])
        .with_width(100)
        .with_alignment(Alignment::Left)
        .with_margin(Margin::new(1, 0, 0, 0)),
    }];

    LayoutSchema::render(
        layout_schemas,
        resume_data,
        data_schemas,
        Path::new("output.txt"),
    )
    .unwrap();
}

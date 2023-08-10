use std::collections::HashMap;

use printpdf::IndirectFontRef;
use printpdf::Mm;
use printpdf::PdfDocument;
use printpdf::PdfLayerReference;

use serde::{Deserialize, Serialize};

use std::{io::ErrorKind, path::Path};

use printpdf;
use std::fs;
use std::io::BufWriter;
use std::io::Error;

use crate::alignment::Alignment;
use crate::data_schema::{DataSchema, Field};
use crate::document::DocumentDefinition;
use crate::font;
use crate::font::FontDict;
use crate::layout_schema::LayoutSchema;
use crate::layout::Layout;
use crate::margin::Margin;
use crate::point::Point;
use crate::element::Element;
use crate::resume_data::{ItemContent, ResumeData};
use crate::spatial_box::SpatialBox;

use rusttype::{point, Font as RFont, Scale};

pub struct PdfLayout {
    pub doc: DocumentDefinition,
}

impl PdfLayout {
    pub fn render(
        &self,
        layout_schemas: Vec<LayoutSchema>,
        resume_data: ResumeData,
        data_schemas: Vec<DataSchema>,
        filepath: &Path,
    ) -> std::io::Result<()> {
        let (doc, page1, layer1) =
            PdfDocument::new("PDF_Document_title", Mm(612.0), Mm(792.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);

        let exo_font = doc
        .add_external_font(fs::File::open("assets/Exo/static/Exo-Medium.ttf").unwrap())
        .unwrap();

        let exo_bold_font = doc
        .add_external_font(fs::File::open("assets/Exo/static/Exo-Bold.ttf").unwrap())
        .unwrap();

        let mut font_dict : HashMap<String, IndirectFontRef> = HashMap::new();

        font_dict.insert("Exo".to_string(), exo_font);
        font_dict.insert("Exo-Bold".to_string(), exo_bold_font);

        
        let mut height = 0;
        let mut boxes: Vec<(SpatialBox, Element)> = Vec::new();
        
        for section in resume_data.sections {
            // Render Section Header
            // 1. Find the layout schema for the section
            let Some(layout_schema) = layout_schemas
                .iter()
                .find(|&s| s.schema_name == section.layout_schema)
            else {
                return Err(Error::new(ErrorKind::Other, format!("Layout not found for {}", section.layout_schema)));
            };
            // 2. Find the data schema for the section
            let data_schema = data_schemas
                .iter()
                .find(|&s| s.name == section.data_schema)
                .unwrap();
            // 3. Render the header

            let mut result = layout_schema
                .header_layout_schema
                .instantiate(&section.data)
                .propagate_widths()
                .normalize(&self.doc)
                .compute_boxes(height, &self.doc.font_dict);

            height = result.0;
            boxes.append(&mut result.1);

            // Render Section Items
            for item in section.items {
                // 1. Find the layout schema for the section
                let layout_schema = layout_schemas
                    .iter()
                    .find(|&s| s.schema_name == section.layout_schema)
                    .unwrap();
                // 2. Find the data schema for the section
                let data_schema = data_schemas
                    .iter()
                    .find(|&s| s.name == section.data_schema)
                    .unwrap();
                // 3. Render the item
                let mut result = layout_schema
                    .item_layout_schema
                    .instantiate(&item)
                    .propagate_widths()
                    .normalize(&self.doc)
                    .compute_boxes(height, &self.doc.font_dict);
    
                height = result.0;
                boxes.append(&mut result.1);
            }
        }

        // Render the boxes
        for element_box in boxes {
            let (box_, element) = element_box;
            println!("({}, {})({}, {}): {}", box_.top_left.x, box_.top_left.y, box_.bottom_right.x, box_.bottom_right.y, element.item);
            current_layer.use_text(
                element.item,
                (element.font.size * 2.0) as f64,
                Mm(box_.top_left.x.into()),
                Mm((self.doc.height - (box_.top_left.y + element.font.get_height(&self.doc.font_dict) as u32) ).into()),
                &font_dict.get(&element.font.name).unwrap(),
            );
        }

        doc.save(&mut BufWriter::new(fs::File::create(filepath).unwrap()))
            .unwrap();

        // fs::write(filepath, contents)
        Ok(())
    }
}




#[cfg(test)]
mod tests {
    use crate::{data_schema, resume_data, layout_schema};

    use super::*;

    #[test]
    fn test_compute_blueprint() {
        let schema = fs::read_to_string("data/data-schemas.json").unwrap();
        let data_schemas = data_schema::DataSchema::from_json(&schema);
    
        let resume = fs::read_to_string("data/resume2.json").unwrap();
        let resume_data = resume_data::ResumeData::from_json(&resume);
    
        
        let schema = fs::read_to_string("data/layout-schemas.json").unwrap();
        let layout_schemas = layout_schema::LayoutSchema::from_json(&schema);

        let mut font_dict = FontDict::new();

        // This only succeeds if collection consists of one font
        let _font = RFont::try_from_bytes(include_bytes!("../assets/Exo/static/Exo-Medium.ttf") as &[u8]).expect("Error constructing Font");

        font_dict.insert("Exo".to_string(), _font);

        let _font = RFont::try_from_bytes(include_bytes!("../assets/Exo/static/Exo-Bold.ttf") as &[u8]).expect("Error constructing Font");

        font_dict.insert("Exo-Bold".to_string(), _font);

        let pdf_layout = PdfLayout {
            doc: DocumentDefinition {
                width: 612,
                height: 792,
                font_dict,
            },
        };

        pdf_layout.render(
            layout_schemas,
            resume_data,
            data_schemas,
            Path::new("results/output.pdf"),
        )
        .unwrap();
    }
}

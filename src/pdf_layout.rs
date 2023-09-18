use std::vec;

use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::properties::Stretch;
use printpdf::Color;
use printpdf::Line;
use printpdf::LinkAnnotation;
use printpdf::Mm;
use printpdf::PdfDocument;
use printpdf::Rect;
use printpdf::Rgb;

use std::{io::ErrorKind, path::Path};

use printpdf;
use std::fs;
use std::io::BufWriter;
use std::io::Error;

use crate::data_schema::DataSchema;
use crate::document::DocumentDefinition;
use crate::element::Element;
use crate::font::Font;
use crate::font::FontDict;
use crate::font::FontLoader;
use crate::font::FontSource;
use crate::font::LoadedFont;
use crate::layout_schema::LayoutSchema;
use crate::resume_data::ResumeData;
use crate::spatial_box::SpatialBox;

use font_kit::source::SystemSource;

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
        debug: bool,
    ) -> std::io::Result<()> {
        let (doc, page1, layer1) =
            PdfDocument::new("PDF_Document_title", Mm(612.0), Mm(792.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);

        let mut font_dict: FontDict = FontDict::new();

        log::info!("Discovering system fonts...");

        let ss = SystemSource::new();
        let system_fonts = ss.all_families().unwrap();

        log::info!("{} system fonts have been discovered!", system_fonts.len());

        log::info!("Discovering locally installed fonts...");

        let local_fonts: Vec<String> = fs::read_dir("assets")
            .unwrap()
            .into_iter()
            .filter_map(|t| t.ok())
            .filter(|dir_entry| dir_entry.file_type().map_or(false, |f| f.is_dir()))
            .map(|dir_entry| dir_entry.file_name().into_string())
            .filter_map(|t| t.ok())
            .collect();

        log::info!("{} local fonts have been discovered!", local_fonts.len());

        let mut height = 0.0;
        let mut boxes: Vec<(SpatialBox, Element)> = Vec::new();

        for section in resume_data.sections {
            // Render Section Header
            // 1. Find the layout schema for the section
            log::info!("Computing section: {}", section.name);

            let Some(layout_schema) = layout_schemas
                .iter()
                .find(|&s| s.schema_name == section.layout_schema)
            else {
                return Err(Error::new(ErrorKind::Other, format!("Layout not found for {}", section.layout_schema)));
            };

            PdfLayout::load_fonts(layout_schema, &doc, &mut font_dict);

            // 2. Find the data schema for the section
            let _data_schema = data_schemas
                .iter()
                .find(|&s| s.name == section.data_schema)
                .unwrap();
            // 3. Render the header

            let mut result = layout_schema
                .header_layout_schema
                .instantiate(&section.data)
                .normalize(&self.doc, &font_dict)
                .compute_boxes(height, &font_dict);

            height = result.0;
            boxes.append(&mut result.1);

            // Render Section Items
            for item in section.items {
                // 1. Find the layout schema for the section
                let layout_schema = layout_schemas
                    .iter()
                    .find(|&s| s.schema_name == section.layout_schema)
                    .unwrap();

                PdfLayout::load_fonts(layout_schema, &doc, &mut font_dict);

                // 2. Find the data schema for the section
                let _data_schema = data_schemas
                    .iter()
                    .find(|&s| s.name == section.data_schema)
                    .unwrap();
                // 3. Render the item
                let mut result = layout_schema
                    .item_layout_schema
                    .instantiate(&item)
                    .normalize(&self.doc, &font_dict)
                    .compute_boxes(height, &font_dict);

                height = result.0;
                boxes.append(&mut result.1);
            }
        }

        log::info!("Position calculations are completed. Rendering the document...");

        // Render the boxes
        for element_box in boxes {
            let (box_, element) = element_box;
            log::debug!(
                "({}, {})({}, {}): {}",
                box_.top_left.x,
                box_.top_left.y,
                box_.bottom_right.x,
                box_.bottom_right.y,
                element.item
            );

            if debug {
                let points: Vec<(printpdf::Point, bool)> = vec![
                    (
                        printpdf::Point::new(
                            Mm(box_.top_left.x.into()),
                            Mm((self.doc.height - box_.top_left.y).into()),
                        ),
                        false,
                    ),
                    (
                        printpdf::Point::new(
                            Mm(box_.bottom_right.x.into()),
                            Mm((self.doc.height - box_.top_left.y).into()),
                        ),
                        false,
                    ),
                    (
                        printpdf::Point::new(
                            Mm(box_.bottom_right.x.into()),
                            Mm((self.doc.height - box_.bottom_right.y).into()),
                        ),
                        false,
                    ),
                    (
                        printpdf::Point::new(
                            Mm(box_.top_left.x.into()),
                            Mm((self.doc.height - box_.bottom_right.y).into()),
                        ),
                        false,
                    ),
                ];
                let line1 = Line {
                    points: points,
                    is_closed: true,
                    has_fill: false,
                    has_stroke: true,
                    is_clipping_path: false,
                };
                let outline_color = Color::Rgb(Rgb::new(0.4, 0.6, 0.2, None));
                current_layer.set_outline_color(outline_color);
                current_layer.add_shape(line1);
            };

            current_layer.use_text(
                element.item,
                (element.font.size * 2.0) as f64,
                Mm(box_.top_left.x.into()),
                Mm(
                    (self.doc.height - (box_.top_left.y + element.font.get_height(&font_dict)))
                        .into(),
                ),
                &font_dict
                    .get(&element.font.full_name())
                    .unwrap_or_else(|| &font_dict.get(&Font::default().full_name()).unwrap())
                    .printpdf_font,
            );

            if let Some(url) = element.url {
                let rect = Rect::new(
                    Mm(box_.top_left.x.into()),
                    Mm((self.doc.height - box_.bottom_right.y).into()),
                    Mm(box_.bottom_right.x.into()),
                    Mm((self.doc.height - box_.top_left.y).into()),
                );
                current_layer.add_link_annotation(LinkAnnotation::new(
                    rect,
                    Some(printpdf::BorderArray::default()),
                    Some(printpdf::ColorArray::default()),
                    printpdf::Actions::uri(url),
                    Some(printpdf::HighlightingMode::Invert),
                ));
            }
        }

        log::info!("Rendering is completed. Saving the document...");

        doc.save(&mut BufWriter::new(fs::File::create(filepath).unwrap()))
            .unwrap();

        log::info!("Document is saved to {}", filepath.to_str().unwrap());

        Ok(())
    }

    pub fn load_font(font: &Font, doc: &printpdf::PdfDocumentReference, font_dict: &mut FontDict) {
        match font.source {
            FontSource::Local => {
                font_dict.load_from_path(
                    &doc,
                    font.full_name(),
                    format!("assets/{}/static/{}.ttf", font.name, font.full_name()),
                );
            }
            FontSource::System => {
                if let Ok(best_match) = SystemSource::new().select_best_match(
                    &[FamilyName::Title(font.name.clone())],
                    &Properties {
                        style: font.style.clone().into(),
                        weight: font.weight.clone().into(),
                        stretch: Stretch::NORMAL,
                    },
                ) {
                    let font_data = best_match.load().unwrap();
                    let font_stream = font_data.copy_font_data().unwrap().clone();
                    let printpdf_font = doc.add_external_font(font_stream.as_slice()).unwrap();
                    let rusttype_font =
                        rusttype::Font::try_from_vec((*font_stream).clone()).unwrap();
                    font_dict.insert(
                        font.full_name(),
                        LoadedFont {
                            printpdf_font,
                            rusttype_font,
                        },
                    );
                    log::info!("{} will be used in your document", font_data.full_name());
                } else {
                    log::info!(
                        "{} was not found in your system, will use the default font",
                        font.full_name()
                    );

                    if !font_dict.contains_key(&Font::default_name()) {
                        let default_font = Font::default();
                        PdfLayout::load_font(&default_font, &doc, font_dict);
                    }
                }
            }
        }
    }

    pub fn load_fonts(
        layout_schema: &LayoutSchema,
        doc: &printpdf::PdfDocumentReference,
        font_dict: &mut FontDict,
    ) {
        for font in layout_schema.fonts() {
            if !font_dict.contains_key(&font.full_name()) {
                PdfLayout::load_font(&font, &doc, font_dict);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{data_schema, font::FontDict, layout_schema, resume_data};

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
        let _font =
            RFont::try_from_bytes(include_bytes!("../assets/Exo/static/Exo-Medium.ttf") as &[u8])
                .expect("Error constructing Font");

        font_dict.insert("Exo".to_string(), _font);

        let _font =
            RFont::try_from_bytes(include_bytes!("../assets/Exo/static/Exo-Bold.ttf") as &[u8])
                .expect("Error constructing Font");

        let x = RFont::try_from_bytes(include_bytes!("../assets/Exo/static/Exo-Bold.ttf") as &[u8])
            .expect("Error constructing Font");

        x.font_dict.insert("Exo-Bold".to_string(), _font);

        let pdf_layout = PdfLayout {
            doc: DocumentDefinition {
                width: 612.0,
                height: 792.0,
                font_dict,
            },
        };

        pdf_layout
            .render(
                layout_schemas,
                resume_data,
                data_schemas,
                Path::new("results/output.pdf"),
                true,
            )
            .unwrap();
    }
}

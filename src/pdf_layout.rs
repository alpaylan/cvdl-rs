use std::{
    collections::HashMap,
    fs,
    io::{BufWriter, Error, ErrorKind},
    path::Path,
    vec,
};

use font_kit::{
    family_name::FamilyName,
    properties::{Properties, Stretch},
    source::SystemSource,
};

use printpdf::{Color, IndirectFontRef, Line, LinkAnnotation, Mm, PdfDocument, Rect, Rgb};

use crate::{
    data_schema::DataSchema,
    document::DocumentDefinition,
    element::Element,
    font::{Font, FontDict, FontLoadSource, FontLoader, FontSource, LoadedFont},
    layout_schema::LayoutSchema,
    resume_data::ResumeData,
    spatial_box::SpatialBox,
};

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
        let (doc, page1, layer1) = PdfDocument::new(
            "PDF_Document_title",
            Mm(self.doc.width as f64),
            Mm(self.doc.height as f64),
            "Layer 1",
        );
        let current_layer = doc.get_page(page1).get_layer(layer1);

        let mut font_dict: FontDict = FontDict::new();

        log::info!("Discovering system fonts...");

        let ss = SystemSource::new();
        let system_fonts = ss.all_families().unwrap();

        log::info!("{} system fonts have been discovered!", system_fonts.len());

        log::info!("Discovering locally installed fonts...");

        let local_fonts: Vec<String> = fs::read_dir("assets")
            .unwrap()
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
                return Err(Error::new(ErrorKind::Other, format!("SectionLayout not found for {}", section.layout_schema)));
            };

            PdfLayout::load_fonts(layout_schema, &mut font_dict);

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
            for (index, item) in section.items.iter().enumerate() {
                log::info!("Computing item {index}");
                // 1. Find the layout schema for the section
                let layout_schema = layout_schemas
                    .iter()
                    .find(|&s| s.schema_name == section.layout_schema)
                    .unwrap();

                PdfLayout::load_fonts(layout_schema, &mut font_dict);

                // 2. Find the data schema for the section
                let _data_schema = data_schemas
                    .iter()
                    .find(|&s| s.name == section.data_schema)
                    .unwrap();
                // 3. Render the item
                let mut result = layout_schema
                    .item_layout_schema
                    .instantiate(item)
                    .normalize(&self.doc, &font_dict)
                    .compute_boxes(height, &font_dict);

                height = result.0;
                if height > self.doc.height {
                    log::warn!("Page height is exceeded. Multi page documents are not yet supported. Your document will be truncated.");
                }
                boxes.append(&mut result.1);
            }
        }

        log::info!("Position calculations are completed. Rendering the document...");

        let printpdf_font_dict: HashMap<String, IndirectFontRef> = font_dict
            .iter_mut()
            .map(|(k, v)| {
                let pdf_font = match &v.source {
                    FontLoadSource::Local(path) => {
                        log::info!("Loading {} from {}", k, path);
                        doc.add_external_font(fs::File::open(path).unwrap())
                            .unwrap()
                    }
                    FontLoadSource::System(font) => {
                        let font_stream = font.copy_font_data().unwrap();
                        doc.add_external_font(font_stream.as_slice()).unwrap()
                    }
                };

                (k.clone(), pdf_font)
            })
            .collect();

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
                    points,
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
                printpdf_font_dict
                    .get(&element.font.full_name())
                    .unwrap_or_else(|| {
                        printpdf_font_dict
                            .get(&Font::default().full_name())
                            .unwrap()
                    }),
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

    pub fn load_font(font: &Font, font_dict: &mut FontDict) {
        match font.source {
            FontSource::Local => {
                font_dict.load_from_path(
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
                    let font_stream = font_data.copy_font_data().unwrap();
                    let rusttype_font =
                        rusttype::Font::try_from_vec((*font_stream).clone()).unwrap();

                    log::info!("{} will be used in your document", font_data.full_name());

                    font_dict.insert(
                        font.full_name(),
                        LoadedFont {
                            source: FontLoadSource::System(font_data),
                            rusttype_font,
                        },
                    );
                } else {
                    log::info!(
                        "{} was not found in your system, will use the default font",
                        font.full_name()
                    );

                    if !font_dict.contains_key(&Font::default_name()) {
                        let default_font = Font::default();
                        PdfLayout::load_font(&default_font, font_dict);
                    }
                }
            }
        }
    }

    pub fn load_fonts(layout_schema: &LayoutSchema, font_dict: &mut FontDict) {
        for font in layout_schema.fonts() {
            if !font_dict.contains_key(&font.full_name()) {
                PdfLayout::load_font(&font, font_dict);
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

        let pdf_layout = PdfLayout {
            doc: DocumentDefinition {
                width: 612.0,
                height: 792.0,
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

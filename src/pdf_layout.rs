use std::collections::HashMap;
use std::fs::DirEntry;
use std::fs::FileType;
use std::vec;

use font_kit::family_handle::FamilyHandle;
use font_kit::file_type;
use font_kit::handle::Handle;
use printpdf::lopdf::content::Operation;
use printpdf::Actions;
use printpdf::BuiltinFont;
use printpdf::Color;
use printpdf::IndirectFontRef;
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
use crate::font::FontSource;
use crate::layout_schema::LayoutSchema;
use crate::resume_data::ResumeData;
use crate::spatial_box::SpatialBox;
use rusttype::Font as RFont;

use printpdf::lopdf::dictionary;
use printpdf::lopdf::Object;

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

        let mut font_dict: HashMap<String, IndirectFontRef> = HashMap::new();

        let ss = SystemSource::new();
        let font_families = ss.all_families().unwrap();

        log::info!("Discovering system fonts...");

        let system_fonts: HashMap<&str, FamilyHandle> = font_families
            .iter()
            .map(|family_name| (family_name, ss.select_family_by_name(family_name)))
            .filter(|item| item.1.is_ok())
            .map(|item| (item.0.as_str(), item.1.unwrap()))
            .collect();

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

        // for family_name in ss.all_families().unwrap() {
        //     println!("{}", family_name);
        //     if let Ok(family) = ss.select_family_by_name(family_name.as_str()) {
        //         let fonts = family.fonts().iter().map(|handler| {
        //             match handler {
        //                 Handle::Path { path, font_index } => println!("Path"),
        //                 Handle::Memory { bytes, font_index } => println!("Memory"),
        //             };
        //             handler.load()
        //         });
        //         for font in fonts {
        //             if let Ok(font) = font {
        //                 let name = font.postscript_name().unwrap();
        //                 let font = font.copy_font_data().unwrap();
        //                 // let font = RFont::try_from_bytes(font.as_slice())
        //                 //     .unwrap();
        //                 // println!("{:?}", font.v_metrics_unscaled());
        //                 let font = doc.add_external_font(font.as_slice()).unwrap();
        //                 font_dict.insert(name, font);
        //             }
        //         }
        //     }
        // }

        let exo_font = doc
            .add_external_font(fs::File::open("assets/Exo/static/Exo-Medium.ttf").unwrap())
            .unwrap();

        let exo_bold_font = doc
            .add_external_font(fs::File::open("assets/Exo/static/Exo-Bold.ttf").unwrap())
            .unwrap();

        font_dict.insert("Exo".to_string(), exo_font);
        font_dict.insert("Exo-Bold".to_string(), exo_bold_font);

        let mut height = 0.0;
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

                for font in layout_schema.fonts() {
                    if !font_dict.contains_key(&font.full_name()) {
                        
                            match font.source {
                                FontSource::Local => {
                                    let font_ref = doc.add_external_font(
                                        fs::File::open(format!("assets/{}/static/{}.ttf", font.name, font.full_name())).unwrap(),
                                    ).unwrap();

                                    font_dict.insert(font.full_name(),font_ref);
                                }
                                FontSource::System => {
                                    log::error!("Wtf is happening");
                                    if let Some(family) = system_fonts.get(font.name.as_str()) {
                                        let fonts = family.fonts().iter().map(Handle::load);
                                        for font_result in fonts {
                                            if let Ok(font_kit_font) = font_result {
                                                let weight = font_kit_font.properties().weight;
                                                let style = font_kit_font.properties().style;
                                                println!("{}-{:?}-{:?}", font_kit_font.family_name(), weight, style);
                                                let font_kit_font = font_kit_font.copy_font_data().unwrap();
                                                let font_ref = doc.add_external_font(font_kit_font.as_slice()).unwrap();
                                                if font.slope
                                            }
                                        }

                                    } else {    
                                        log::info!("{} was not found in your system, will use the default font", font.name);

                                    }   
                                                    
                                }
                            }
                            
                        println!("Inserted");
                    }
                    println!("{:?}", font.full_name());
                }

                // 2. Find the data schema for the section
                let data_schema = data_schemas
                    .iter()
                    .find(|&s| s.name == section.data_schema)
                    .unwrap();
                // 3. Render the item
                let mut result = layout_schema
                    .item_layout_schema
                    .instantiate(&item)
                    .normalize(&self.doc)
                    .compute_boxes(height, &self.doc.font_dict);

                height = result.0;
                boxes.append(&mut result.1);
            }
        }

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
                Mm((self.doc.height
                    - (box_.top_left.y + element.font.get_height(&self.doc.font_dict)))
                .into()),
                font_dict.get(&element.font.name).unwrap(),
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

        doc.save(&mut BufWriter::new(fs::File::create(filepath).unwrap()))
            .unwrap();

        // fs::write(filepath, contents)
        Ok(())
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

        font_dict.insert("Exo-Bold".to_string(), _font);

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

use std::path::Path;

use image::{DynamicImage, Rgba};
use rusttype::{point, Scale};

use crate::{
    any_layout::AnyLayout, data_schema::DataSchema, font::Font, layout_schema::LayoutSchema,
    resume_data::ResumeData, resume_layout::ResumeLayout,
};

pub struct PngLayout;

impl PngLayout {
    pub fn render(
        layout_schemas: Vec<LayoutSchema>,
        resume_data: ResumeData,
        data_schemas: Vec<DataSchema>,
        resume_layout: ResumeLayout,
        _filepath: &Path,
        _debug: bool,
    ) -> std::io::Result<()> {
        // Create a new rgba image with some padding

        let (font_dict, pages) =
            AnyLayout::render(&layout_schemas, &resume_data, &data_schemas, &resume_layout)
                .unwrap();

        for (index, page) in pages.iter().enumerate() {
            let mut image =
                DynamicImage::new_rgba8(resume_layout.width as u32, resume_layout.height as u32)
                    .to_rgba8();

            for element_box in page {
                for element in &element_box.elements {
                    let text = &element.1.item;
                    let font = &element.1.font;
                    let scale = Scale::uniform(font.size);
                    let font = &font_dict
                        .get(&font.full_name())
                        .unwrap_or_else(|| font_dict.get(&Font::default().full_name()).unwrap())
                        .rusttype_font;
                    let v_metrics = font.v_metrics(scale);

                    // layout the glyphs in a line with 20 pixels padding
                    let glyphs: Vec<_> = font
                        .layout(text, scale, point(0.0, v_metrics.ascent))
                        .collect();

                    for glyph in glyphs {
                        if let Some(bounding_box) = glyph.pixel_bounding_box() {
                            // Draw the glyph into the image per-pixel by using the draw closure
                            glyph.draw(|x, y, v| {
                                image.put_pixel(
                                    // Offset the position by the glyph bounding box
                                    element.0.top_left.x as u32 + x + bounding_box.min.x as u32,
                                    element.0.top_left.y as u32 + y + bounding_box.min.y as u32,
                                    // Turn the coverage into an alpha value
                                    Rgba([0, 0, 0, (v * 255.0) as u8]),
                                )
                            });
                        }
                    }
                }
            }
            image.save(format!("output_{}.png", index)).unwrap();
            println!("{}", format!("Generated: output_{}.png", index));
        }
        Ok(())
    }
}

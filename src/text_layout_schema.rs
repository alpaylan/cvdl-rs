use std::{io::ErrorKind, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    data_schema::DataSchema,
    resume_data::{ItemContent, ResumeData},
};

use std::fs;
use std::io::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct TextLayoutSchema {
    #[serde(rename = "schema-name")]
    pub schema_name: String,
    #[serde(rename = "header-layout-schema")]
    pub header_layout_schema: String,
    #[serde(rename = "item-layout-schema")]
    pub item_layout_schema: String,
}

impl TextLayoutSchema {
    pub fn from_json(json: &str) -> Vec<TextLayoutSchema> {
        let schema: Vec<TextLayoutSchema> = serde_json::from_str(json).unwrap();
        schema
    }
}

impl TextLayoutSchema {
    pub fn render(
        layout_schemas: Vec<TextLayoutSchema>,
        resume_data: ResumeData,
        data_schemas: Vec<DataSchema>,
        filepath: &Path,
    ) -> std::io::Result<()> {
        let mut contents = String::new();

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
            let mut header_blueprint = layout_schema.header_layout_schema.clone();
            for field in &data_schema.header_schema {
                let field_name = format!("{{{}}}", field.name);
                let field_value = section.data.get(&field.name).unwrap();
                header_blueprint = header_blueprint.replace(&field_name, &field_value.to_string());
            }
            contents.push_str(&header_blueprint);

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
                let mut item_blueprint = layout_schema.item_layout_schema.clone();
                for field in &data_schema.item_schema {
                    if item_blueprint.contains(&field.name) == false {
                        continue;
                    }

                    let field_name = format!("{{{}}}", field.name);
                    let field_value = item.get(&field.name).unwrap_or(&ItemContent::None);
                    item_blueprint = item_blueprint.replace(&field_name, &field_value.to_string());
                }
                contents.push_str(&item_blueprint);
            }
        }

        fs::write(filepath, contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_json() {
        let json = r#"
        [
    {
        "schema-name": "Education",
        "header-layout-schema": "{Title}\n\n",
        "item-layout-schema": "{School} - {Degree} \t\t {Date-Started} - {Date-Finished} \n {Department} \t\t {Location} \n {Text}"
    },
    {
        "schema-name": "General",
        "header-layout-schema": "{Title}\n\n",
        "item-layout-schema": "{Title} \t\t {Date-Started} - {Date-Finished} \n {Text}"
    },
    {
        "schema-name": "Skill",
        "header-layout-schema": "{Title}\n\n",
        "item-layout-schema": "{Skill} \t\t {Level} \n {Information}"
    },
    {
        "schema-name": "Profile",
        "header-layout-schema": "{Title}\n\n",
        "item-layout-schema": "{Name} {Surname} \n {Linkedin} \n {Github} \n {Email} \n {Google Scholar}"
    }
]
        "#;

        let schema = TextLayoutSchema::from_json(json);
        assert_eq!(schema[0].schema_name, "Education");
        assert_eq!(schema[0].item_layout_schema, "{School} - {Degree} \t\t {Date-Started} - {Date-Finished} \n {Department} \t\t {Location} \n {Text}");
    }
}

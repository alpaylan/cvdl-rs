
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum DocumentDataType {
    Type(String),
    Types(Vec<DocumentDataType>),
}

impl DocumentDataType {
    pub fn from_str(s: &str) -> DocumentDataType {
        if s.contains("|") {
            let types: Vec<&str> = s.split("|").collect();
            let mut data_types: Vec<DocumentDataType> = Vec::new();
            for t in types {
                data_types.push(DocumentDataType::Type(t.trim().to_string()));
            }
            DocumentDataType::Types(data_types)
        } else {
            DocumentDataType::Type(s.to_string())
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: DocumentDataType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DataSchema {
    #[serde(rename = "schema-name")]
    pub name: String,
    #[serde(rename = "element-schema")]
    pub fields: Vec<Field>,
}

impl DataSchema {

    pub fn from_json(json: &str) -> Vec<DataSchema> {
        let schema: Vec<DataSchema> = serde_json::from_str(json).unwrap();
        schema
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
                "element-schema": [
                    { "name": "School", "type": "String" },
                    { "name": "Degree", "type": "String" },
                    { "name": "Department", "type": "String" },
                    { "name": "Date-Started", "type": "Date" },
                    { "name": "Date-Finished", "type": ["Date", "String"] },
                    { "name": "Location", "type": "String" },
                    { "name": "Text", "type": "MarkdownString" }
                ]
            },
            {
                "schema-name": "General",
                "element-schema": [
                    { "name": "Title", "type": "String" },
                    { "name": "Subtitle", "type": "String" },
                    { "name": "Text", "type": "String" },
                    { "name": "Date-Started", "type": "Date" },
                    { "name": "Date-Finished", "type": ["Date", "String"] },
                    { "name": "Location", "type": "String" },
                    { "name": "Text", "type": "MarkdownString" }
                ]
            },
            {
                "schema-name": "Skill",
                "element-schema": [
                    { "name": "Skill", "type": "String" },
                    { "name": "Information", "type": "String" },
                    { "name": "Level", "type": "Number" }
                ]
            },
            {
                "schema-name": "Profile",
                "element-schema": [
                    { "name": "Name", "type": "String" },
                    { "name": "Surname", "type": "String" },
                    { "name": "Linkedin", "type": "String" },
                    { "name": "Github", "type": "String" },
                    { "name": "Email", "type": "String" },
                    { "name": "Google Scholar", "type": "String" }
                ]
            }
        ]
        "#;
        let schema = DataSchema::from_json(json);
        assert_eq!(schema[0].name, "Education");
        assert_eq!(schema[0].fields[0].name, "School");
        assert_eq!(schema[0].fields[0].data_type, DocumentDataType::from_str("String"));
        assert_eq!(schema[0].fields[1].name, "Degree");
        assert_eq!(schema[0].fields[1].data_type, DocumentDataType::from_str("String"));
        assert_eq!(schema[0].fields[4].name, "Date-Finished");
        assert_eq!(schema[0].fields[4].data_type, DocumentDataType::from_str("Date | String"));
    }
}

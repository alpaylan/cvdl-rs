
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TextLayoutSchema {
    #[serde(rename = "schema-name")]
    pub schema_name: String,
    #[serde(rename = "text-layout-schema")]
    pub text_layout_schema: String
}

impl TextLayoutSchema {

    pub fn from_json(json: &str) -> Vec<TextLayoutSchema> {
        let schema: Vec<TextLayoutSchema> = serde_json::from_str(json).unwrap();
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
        "text-layout-schema": "{School} - {Degree} \t\t {Date-Started} - {Date-Finished} \n {Department} \t\t {Location} \n {Text}"
    },
    {
        "schema-name": "General",
        "text-layout-schema": "{Title} \t\t {Date-Started} - {Date-Finished} \n {Text}"
    },
    {
        "schema-name": "Skill",
        "text-layout-schema": "{Skill} \t\t {Level} \n {Information}"
    },
    {
        "schema-name": "Profile",
        "text-layout-schema": "{Name} {Surname} \n {Linkedin} \n {Github} \n {Email} \n {Google Scholar}"
    }
]
        "#;

        let schema = TextLayoutSchema::from_json(json);
        assert_eq!(schema[0].schema_name, "Education");
        assert_eq!(schema[0].text_layout_schema, "{School} - {Degree} \t\t {Date-Started} - {Date-Finished} \n {Department} \t\t {Location} \n {Text}");
    }
}
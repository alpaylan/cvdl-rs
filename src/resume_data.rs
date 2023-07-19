use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_with::serde_as;

#[derive(Serialize, Deserialize, Debug)]
pub struct ResumeData {
    pub sections: Vec<ResumeSection>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct ResumeSection {
    #[serde(rename = "section-name")]
    pub name: String,
    pub schema: String,
    #[serde_as(deserialize_as = "Vec<HashMap<_, _>>")]
    pub elements: Vec<HashMap<ElementName, ElementContent>>,
}

pub type ElementName = String;
pub type ElementContent = String;

impl ResumeData {

    pub fn from_json(json: &str) -> ResumeData {
        let sections: Vec<ResumeSection> = serde_json::from_str(json).unwrap();
        ResumeData { sections }
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
        "section-name": "Profile",
        "schema": "Profile",
        "elements": [
            { 
                "Name": "Alperen",
                "Surname": "Keles",
                "Linkedin": "https://linkedin.com/in/alpkeles",
                "Github": "https://github.com/alpaylan",
                "Email": "alpkeles99@gmail.com",
                "Google Scholar": "https://scholar.google.com/citations?user=5T4PvEAAAAAJ&hl=tr" 
            }
        ]
    },
    {
        "section-name": "Education",
        "schema": "Education",
        "elements": [
            { 
                "School": "University of Maryland, College Park",
                "Degree": "Doctorate of Philosophy",
                "Department": "Computer Science",
                "Date-Started": "2021",
                "Date-Finished": "2026(Expected)",
                "Location": "Maryland, USA" 
            },
            { 
                "School": "Middle East Technical University",
                "Degree": "Bachelor of Engineering",
                "Department": "Computer Engineering",
                "Date-Started": "2017",
                "Date-Finished": "2021",
                "Location": "Ankara, Turkey",
                "Text": "GPA: 3.66/4.0 ***(top 5% in class of 229)***"
            }
        ]
    }
]
        "#;

    let resume_data = ResumeData::from_json(json);
    assert_eq!(resume_data.sections.len(), 2);
    assert_eq!(resume_data.sections[0].name, "Profile");
    assert_eq!(resume_data.sections[0].schema, "Profile");
    assert_eq!(resume_data.sections[0].elements.len(), 1);
    assert_eq!(resume_data.sections[0].elements[0]["Name"], "Alperen");
    assert_eq!(resume_data.sections[1].name, "Education");
    assert_eq!(resume_data.sections[1].schema, "Education");
    assert_eq!(resume_data.sections[1].elements.len(), 2);
    assert_eq!(resume_data.sections[1].elements[0]["School"], "University of Maryland, College Park");
    }
}
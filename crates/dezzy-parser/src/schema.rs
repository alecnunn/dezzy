use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YamlFormat {
    pub name: String,
    pub version: Option<String>,
    pub endianness: Option<String>,
    pub types: Vec<YamlTypeDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YamlTypeDef {
    pub name: String,
    #[serde(rename = "type")]
    pub type_kind: String,
    pub doc: Option<String>,
    pub fields: Option<Vec<YamlField>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YamlField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub doc: Option<String>,
    pub until: Option<String>,
}

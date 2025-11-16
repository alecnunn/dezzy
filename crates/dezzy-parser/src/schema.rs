use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YamlFormat {
    pub name: String,
    pub version: Option<String>,
    pub endianness: Option<String>,
    pub bit_order: Option<String>,
    #[serde(default)]
    pub enums: Vec<YamlEnum>,
    pub types: Vec<YamlTypeDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YamlEnum {
    pub name: String,
    #[serde(rename = "type")]
    pub underlying_type: String,
    pub doc: Option<String>,
    pub values: serde_yaml::Mapping,
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
    #[serde(rename = "assert")]
    pub assertion: Option<serde_yaml::Value>,
    pub skip: Option<String>,
    pub padding: Option<usize>,
    pub align: Option<usize>,
}

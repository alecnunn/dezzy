use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Invalid type reference: {0}")]
    InvalidTypeReference(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid value for field {field}: {message}")]
    InvalidValue { field: String, message: String },

    #[error("Duplicate type definition: {0}")]
    DuplicateType(String),

    #[error("Unknown type: {0}")]
    UnknownType(String),
}

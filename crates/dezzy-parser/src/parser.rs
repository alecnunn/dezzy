use crate::error::ParseError;
use crate::schema::{YamlField, YamlFormat, YamlTypeDef};
use dezzy_core::hir::{Endianness, HirField, HirFormat, HirStruct, HirType, HirTypeDef};
use std::collections::HashSet;

pub fn parse_format(yaml_content: &str) -> Result<HirFormat, ParseError> {
    let yaml_format: YamlFormat = serde_yaml::from_str(yaml_content)?;

    let endianness = parse_endianness(yaml_format.endianness.as_deref())?;

    let mut type_names = HashSet::new();
    for type_def in &yaml_format.types {
        if !type_names.insert(type_def.name.clone()) {
            return Err(ParseError::DuplicateType(type_def.name.clone()));
        }
    }

    let mut hir_types = Vec::new();
    for type_def in &yaml_format.types {
        let hir_type = parse_type_def(type_def, &type_names)?;
        hir_types.push(hir_type);
    }

    Ok(HirFormat {
        name: yaml_format.name,
        version: yaml_format.version,
        endianness,
        types: hir_types,
    })
}

fn parse_endianness(endianness: Option<&str>) -> Result<Endianness, ParseError> {
    match endianness {
        None | Some("little") => Ok(Endianness::Little),
        Some("big") => Ok(Endianness::Big),
        Some("native") => Ok(Endianness::Native),
        Some(other) => Err(ParseError::InvalidValue {
            field: "endianness".to_string(),
            message: format!("Unknown endianness '{}', expected 'little', 'big', or 'native'", other),
        }),
    }
}

fn parse_type_def(
    type_def: &YamlTypeDef,
    known_types: &HashSet<String>,
) -> Result<HirTypeDef, ParseError> {
    match type_def.type_kind.as_str() {
        "struct" => {
            let fields = type_def
                .fields
                .as_ref()
                .ok_or_else(|| ParseError::MissingField("fields".to_string()))?;

            let hir_fields = fields
                .iter()
                .map(|f| parse_field(f, known_types))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(HirTypeDef::Struct(HirStruct {
                name: type_def.name.clone(),
                doc: type_def.doc.clone(),
                fields: hir_fields,
            }))
        }
        other => Err(ParseError::InvalidValue {
            field: "type".to_string(),
            message: format!("Unknown type kind '{}', expected 'struct'", other),
        }),
    }
}

fn parse_field(field: &YamlField, known_types: &HashSet<String>) -> Result<HirField, ParseError> {
    let field_type = parse_type(&field.field_type, known_types)?;

    Ok(HirField {
        name: field.name.clone(),
        doc: field.doc.clone(),
        field_type,
    })
}

fn parse_type(type_str: &str, known_types: &HashSet<String>) -> Result<HirType, ParseError> {
    if let Some(array_type) = parse_array_type(type_str)? {
        let (element_type_str, size) = array_type;
        let element_type = parse_type(&element_type_str, known_types)?;
        return Ok(HirType::Array {
            element_type: Box::new(element_type),
            size,
        });
    }

    Ok(match type_str {
        "u8" => HirType::U8,
        "u16" => HirType::U16,
        "u32" => HirType::U32,
        "u64" => HirType::U64,
        "i8" => HirType::I8,
        "i16" => HirType::I16,
        "i32" => HirType::I32,
        "i64" => HirType::I64,
        other => {
            if known_types.contains(other) {
                HirType::UserDefined(other.to_string())
            } else {
                return Err(ParseError::UnknownType(other.to_string()));
            }
        }
    })
}

fn parse_array_type(type_str: &str) -> Result<Option<(String, usize)>, ParseError> {
    if let Some(bracket_pos) = type_str.find('[') {
        if !type_str.ends_with(']') {
            return Err(ParseError::InvalidValue {
                field: "type".to_string(),
                message: format!("Invalid array syntax: {}", type_str),
            });
        }

        let element_type = type_str[..bracket_pos].to_string();
        let size_str = &type_str[bracket_pos + 1..type_str.len() - 1];
        let size = size_str.parse::<usize>().map_err(|_| ParseError::InvalidValue {
            field: "type".to_string(),
            message: format!("Invalid array size: {}", size_str),
        })?;

        Ok(Some((element_type, size)))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_format() {
        let yaml = r#"
name: TestFormat
version: "1.0"
endianness: little
types:
  - name: Header
    type: struct
    fields:
      - name: magic
        type: u32
      - name: version
        type: u16
"#;

        let result = parse_format(yaml);
        assert!(result.is_ok());

        let format = result.unwrap();
        assert_eq!(format.name, "TestFormat");
        assert_eq!(format.version, Some("1.0".to_string()));
        assert_eq!(format.endianness, Endianness::Little);
        assert_eq!(format.types.len(), 1);
    }

    #[test]
    fn test_parse_array_type() {
        let result = parse_array_type("u8[16]");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(("u8".to_string(), 16)));
    }
}

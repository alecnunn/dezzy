use crate::error::ParseError;
use crate::expr_parser::parse_expr;
use crate::schema::{YamlEnum, YamlField, YamlFormat, YamlTypeDef};
use dezzy_core::hir::{
    Endianness, HirAssertion, HirAssertValue, HirEnum, HirEnumValue, HirField, HirFormat,
    HirPrimitiveType, HirStruct, HirType, HirTypeDef,
};
use std::collections::HashSet;

pub fn parse_format(yaml_content: &str) -> Result<HirFormat, ParseError> {
    let yaml_format: YamlFormat = serde_yaml::from_str(yaml_content)?;

    let endianness = parse_endianness(yaml_format.endianness.as_deref())?;

    // Parse enums first
    let mut enum_names = HashSet::new();
    let mut hir_enums = Vec::new();
    for enum_def in &yaml_format.enums {
        if !enum_names.insert(enum_def.name.clone()) {
            return Err(ParseError::DuplicateType(enum_def.name.clone()));
        }
        let hir_enum = parse_enum(enum_def)?;
        hir_enums.push(hir_enum);
    }

    // Collect all known type names (enums + structs)
    let mut type_names = enum_names.clone();
    for type_def in &yaml_format.types {
        if !type_names.insert(type_def.name.clone()) {
            return Err(ParseError::DuplicateType(type_def.name.clone()));
        }
    }

    let mut hir_types = Vec::new();
    for type_def in &yaml_format.types {
        let hir_type = parse_type_def(type_def, &type_names, &enum_names)?;
        hir_types.push(hir_type);
    }

    Ok(HirFormat {
        name: yaml_format.name,
        version: yaml_format.version,
        endianness,
        enums: hir_enums,
        types: hir_types,
    })
}

fn parse_enum(enum_def: &YamlEnum) -> Result<HirEnum, ParseError> {
    let underlying_type = parse_primitive_type(&enum_def.underlying_type)?;

    let mut values = Vec::new();
    for (key, value) in &enum_def.values {
        let name = key
            .as_str()
            .ok_or_else(|| ParseError::InvalidValue {
                field: "enum value name".to_string(),
                message: "Enum value names must be strings".to_string(),
            })?
            .to_string();

        let value_int = if let Some(v) = value.as_i64() {
            v
        } else if let Some(v) = value.as_u64() {
            v as i64
        } else {
            return Err(ParseError::InvalidValue {
                field: format!("enum value '{}'", name),
                message: "Enum values must be integers".to_string(),
            });
        };

        values.push(HirEnumValue {
            name,
            value: value_int,
            doc: None,
        });
    }

    Ok(HirEnum {
        name: enum_def.name.clone(),
        doc: enum_def.doc.clone(),
        underlying_type,
        values,
    })
}

fn parse_primitive_type(type_str: &str) -> Result<HirPrimitiveType, ParseError> {
    match type_str {
        "u8" => Ok(HirPrimitiveType::U8),
        "u16" => Ok(HirPrimitiveType::U16),
        "u32" => Ok(HirPrimitiveType::U32),
        "u64" => Ok(HirPrimitiveType::U64),
        "i8" => Ok(HirPrimitiveType::I8),
        "i16" => Ok(HirPrimitiveType::I16),
        "i32" => Ok(HirPrimitiveType::I32),
        "i64" => Ok(HirPrimitiveType::I64),
        _ => Err(ParseError::InvalidValue {
            field: "type".to_string(),
            message: format!("Unknown primitive type '{}'", type_str),
        }),
    }
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
    enum_names: &HashSet<String>,
) -> Result<HirTypeDef, ParseError> {
    match type_def.type_kind.as_str() {
        "struct" => {
            let fields = type_def
                .fields
                .as_ref()
                .ok_or_else(|| ParseError::MissingField("fields".to_string()))?;

            let hir_fields = fields
                .iter()
                .map(|f| parse_field(f, known_types, enum_names))
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

fn parse_field(
    field: &YamlField,
    known_types: &HashSet<String>,
    enum_names: &HashSet<String>,
) -> Result<HirField, ParseError> {
    let field_type = parse_type(&field.field_type, known_types, enum_names, field.until.as_deref())?;
    let assertion = if let Some(ref assert_value) = field.assertion {
        Some(parse_assertion(assert_value, &field.name)?)
    } else {
        None
    };

    Ok(HirField {
        name: field.name.clone(),
        doc: field.doc.clone(),
        field_type,
        assertion,
        skip: field.skip.clone(),
    })
}

fn parse_assertion(
    value: &serde_yaml::Value,
    field_name: &str,
) -> Result<HirAssertion, ParseError> {
    // Handle mapping format: { equals: 42 } or { in: [1, 2, 3] }
    if let Some(mapping) = value.as_mapping() {
        if mapping.len() != 1 {
            return Err(ParseError::InvalidValue {
                field: format!("assert for field '{}'", field_name),
                message: "Assertion must have exactly one operation".to_string(),
            });
        }

        let (key, val) = mapping.iter().next().unwrap();
        let key_str = key.as_str().ok_or_else(|| ParseError::InvalidValue {
            field: format!("assert for field '{}'", field_name),
            message: "Assertion key must be a string".to_string(),
        })?;

        match key_str {
            "equals" => {
                let assert_val = parse_assert_value(val, field_name)?;
                Ok(HirAssertion::Equals(assert_val))
            }
            "not_equals" => {
                let assert_val = parse_assert_value(val, field_name)?;
                Ok(HirAssertion::NotEquals(assert_val))
            }
            "greater_than" => {
                let threshold = parse_int(val, field_name, "greater_than")?;
                Ok(HirAssertion::GreaterThan(threshold))
            }
            "greater_or_equal" => {
                let threshold = parse_int(val, field_name, "greater_or_equal")?;
                Ok(HirAssertion::GreaterOrEqual(threshold))
            }
            "less_than" => {
                let threshold = parse_int(val, field_name, "less_than")?;
                Ok(HirAssertion::LessThan(threshold))
            }
            "less_or_equal" => {
                let threshold = parse_int(val, field_name, "less_or_equal")?;
                Ok(HirAssertion::LessOrEqual(threshold))
            }
            "in" => {
                let values = parse_int_list(val, field_name, "in")?;
                Ok(HirAssertion::In(values))
            }
            "not_in" => {
                let values = parse_int_list(val, field_name, "not_in")?;
                Ok(HirAssertion::NotIn(values))
            }
            "range" => {
                if let Some(seq) = val.as_sequence() {
                    if seq.len() != 2 {
                        return Err(ParseError::InvalidValue {
                            field: format!("assert.range for field '{}'", field_name),
                            message: "Range must have exactly two values [min, max]".to_string(),
                        });
                    }
                    let min = parse_int(&seq[0], field_name, "range[0]")?;
                    let max = parse_int(&seq[1], field_name, "range[1]")?;
                    Ok(HirAssertion::Range { min, max })
                } else {
                    Err(ParseError::InvalidValue {
                        field: format!("assert.range for field '{}'", field_name),
                        message: "Range must be a sequence [min, max]".to_string(),
                    })
                }
            }
            _ => Err(ParseError::InvalidValue {
                field: format!("assert for field '{}'", field_name),
                message: format!("Unknown assertion type '{}'", key_str),
            }),
        }
    } else {
        Err(ParseError::InvalidValue {
            field: format!("assert for field '{}'", field_name),
            message: "Assertion must be a mapping (e.g., { equals: 42 })".to_string(),
        })
    }
}

fn parse_assert_value(
    value: &serde_yaml::Value,
    field_name: &str,
) -> Result<HirAssertValue, ParseError> {
    if let Some(seq) = value.as_sequence() {
        // Array of integers
        let mut values = Vec::new();
        for item in seq {
            values.push(parse_int(item, field_name, "array element")?);
        }
        Ok(HirAssertValue::IntArray(values))
    } else {
        // Single integer
        let int_val = parse_int(value, field_name, "value")?;
        Ok(HirAssertValue::Int(int_val))
    }
}

fn parse_int(
    value: &serde_yaml::Value,
    field_name: &str,
    context: &str,
) -> Result<i64, ParseError> {
    if let Some(i) = value.as_i64() {
        Ok(i)
    } else if let Some(u) = value.as_u64() {
        Ok(u as i64)
    } else {
        Err(ParseError::InvalidValue {
            field: format!("{} for field '{}'", context, field_name),
            message: "Value must be an integer".to_string(),
        })
    }
}

fn parse_int_list(
    value: &serde_yaml::Value,
    field_name: &str,
    context: &str,
) -> Result<Vec<i64>, ParseError> {
    if let Some(seq) = value.as_sequence() {
        let mut values = Vec::new();
        for item in seq {
            values.push(parse_int(item, field_name, context)?);
        }
        Ok(values)
    } else {
        Err(ParseError::InvalidValue {
            field: format!("{} for field '{}'", context, field_name),
            message: "Value must be a sequence".to_string(),
        })
    }
}

fn parse_type(
    type_str: &str,
    known_types: &HashSet<String>,
    enum_names: &HashSet<String>,
    until: Option<&str>,
) -> Result<HirType, ParseError> {
    // Handle null-terminated string
    if type_str == "cstr" {
        return Ok(HirType::NullTerminatedString);
    }

    // Handle length-prefixed string: str(field_name)
    if type_str.starts_with("str(") && type_str.ends_with(')') {
        let length_field = type_str[4..type_str.len() - 1].to_string();
        return Ok(HirType::LengthPrefixedString { length_field });
    }

    // Handle blob: blob(size_field)
    if type_str.starts_with("blob(") && type_str.ends_with(')') {
        let size_field = type_str[5..type_str.len() - 1].to_string();
        return Ok(HirType::Blob { size_field });
    }

    if let Some((element_type_str, size_spec)) = parse_array_type(type_str)? {
        // Special case: str[N] is a fixed-length string, not an array of bytes
        if element_type_str == "str" {
            if let Ok(size) = size_spec.parse::<usize>() {
                return Ok(HirType::FixedString { size });
            } else {
                return Err(ParseError::InvalidValue {
                    field: "type".to_string(),
                    message: format!("Fixed-length string must have numeric size, got '{}'", size_spec),
                });
            }
        }

        let element_type = parse_type(&element_type_str, known_types, enum_names, None)?;

        // Check if size_spec is empty (for Type[])
        if size_spec.is_empty() {
            // Check for until clause
            if let Some(until_str) = until {
                if until_str == "eof" {
                    // Until-EOF array
                    return Ok(HirType::UntilEofArray {
                        element_type: Box::new(element_type),
                    });
                } else {
                    // Until-condition array (parse expression)
                    let condition = parse_expr(until_str)?;
                    return Ok(HirType::UntilConditionArray {
                        element_type: Box::new(element_type),
                        condition,
                    });
                }
            } else {
                return Err(ParseError::InvalidValue {
                    field: "type".to_string(),
                    message: "Array with empty size [] requires an 'until' clause (e.g., 'until: eof' or 'until: <expression>')".to_string(),
                });
            }
        }

        // Check if size_spec is a number or a field reference
        if let Ok(size) = size_spec.parse::<usize>() {
            // Fixed-size array
            return Ok(HirType::Array {
                element_type: Box::new(element_type),
                size,
            });
        } else {
            // Dynamic array (size from field)
            return Ok(HirType::DynamicArray {
                element_type: Box::new(element_type),
                size_field: size_spec,
            });
        }
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
            if enum_names.contains(other) {
                HirType::Enum(other.to_string())
            } else if known_types.contains(other) {
                HirType::UserDefined(other.to_string())
            } else {
                return Err(ParseError::UnknownType(other.to_string()));
            }
        }
    })
}

fn parse_array_type(type_str: &str) -> Result<Option<(String, String)>, ParseError> {
    if let Some(bracket_pos) = type_str.find('[') {
        if !type_str.ends_with(']') {
            return Err(ParseError::InvalidValue {
                field: "type".to_string(),
                message: format!("Invalid array syntax: {}", type_str),
            });
        }

        let element_type = type_str[..bracket_pos].to_string();
        let size_str = type_str[bracket_pos + 1..type_str.len() - 1].to_string();

        Ok(Some((element_type, size_str)))
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
        assert_eq!(result.unwrap(), Some(("u8".to_string(), "16".to_string())));

        let result2 = parse_array_type("u8[length]");
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), Some(("u8".to_string(), "length".to_string())));
    }
}

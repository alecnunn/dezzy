use dezzy_core::pipeline::Pipeline;
use dezzy_parser::parse_format;

#[test]
fn test_simple_format_parsing() {
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

    let format = result.expect("parse_format should succeed (checked above)");
    assert_eq!(format.name, "TestFormat");
    assert_eq!(format.version, Some("1.0".to_string()));
    assert_eq!(format.types.len(), 1);
}

#[test]
fn test_pipeline_lowering() {
    let yaml = r#"
name: SimpleStruct
endianness: little
types:
  - name: Data
    type: struct
    fields:
      - name: value
        type: u32
"#;

    let hir_format = parse_format(yaml).expect("Failed to parse");
    let mut pipeline = Pipeline::new();
    let lir_format = pipeline.lower(hir_format).expect("Failed to lower");

    assert_eq!(lir_format.name, "SimpleStruct");
    assert_eq!(lir_format.types.len(), 1);
    assert_eq!(lir_format.types[0].name, "Data");
}

#[test]
fn test_array_parsing() {
    let yaml = r#"
name: ArrayTest
endianness: little
types:
  - name: Block
    type: struct
    fields:
      - name: data
        type: u8[16]
"#;

    let result = parse_format(yaml);
    assert!(result.is_ok());

    let format = result.expect("parse_format should succeed (checked above)");
    assert_eq!(format.types.len(), 1);
}

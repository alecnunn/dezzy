use crate::templates;
use anyhow::Result;
use dezzy_backend::{Backend, GeneratedCode, GeneratedFile};
use dezzy_core::hir::Endianness;
use dezzy_core::lir::{LirFormat, LirOperation, LirType};

pub struct CppBackend;

impl CppBackend {
    pub fn new() -> Self {
        Self
    }

    fn generate_type(&self, lir_type: &LirType, endianness: Endianness) -> Result<String> {
        let fields = self.extract_fields(lir_type)?;
        let mut code = templates::generate_struct_declaration(&lir_type.name, &fields);

        code.push_str(&self.generate_read_impl(lir_type, endianness)?);
        code.push_str(&self.generate_write_impl(lir_type, endianness)?);

        Ok(code)
    }

    fn extract_fields(&self, lir_type: &LirType) -> Result<Vec<(String, String)>> {
        let mut fields = Vec::new();
        let mut field_index = 0;

        for op in &lir_type.operations {
            if let Some((field_name, field_type)) = self.operation_to_field(op, field_index) {
                fields.push((field_name, field_type));
                field_index += 1;
            }

            if matches!(op, LirOperation::CreateStruct { .. }) {
                break;
            }
        }

        Ok(fields)
    }

    fn operation_to_field(&self, op: &LirOperation, field_index: usize) -> Option<(String, String)> {
        let field_name = format!("field_{}", field_index);

        let field_type = match op {
            LirOperation::ReadU8 { .. } => "uint8_t",
            LirOperation::ReadU16 { .. } => "uint16_t",
            LirOperation::ReadU32 { .. } => "uint32_t",
            LirOperation::ReadU64 { .. } => "uint64_t",
            LirOperation::ReadI8 { .. } => "int8_t",
            LirOperation::ReadI16 { .. } => "int16_t",
            LirOperation::ReadI32 { .. } => "int32_t",
            LirOperation::ReadI64 { .. } => "int64_t",
            LirOperation::ReadArray { element_op, count, .. } => {
                let element_type = self.get_element_type(element_op);
                return Some((field_name, format!("std::array<{}, {}>", element_type, count)));
            }
            LirOperation::ReadStruct { type_name, .. } => {
                return Some((field_name, type_name.clone()));
            }
            _ => return None,
        };

        Some((field_name, field_type.to_string()))
    }

    fn get_element_type(&self, op: &LirOperation) -> String {
        match op {
            LirOperation::ReadU8 { .. } => "uint8_t".to_string(),
            LirOperation::ReadU16 { .. } => "uint16_t".to_string(),
            LirOperation::ReadU32 { .. } => "uint32_t".to_string(),
            LirOperation::ReadU64 { .. } => "uint64_t".to_string(),
            LirOperation::ReadI8 { .. } => "int8_t".to_string(),
            LirOperation::ReadI16 { .. } => "int16_t".to_string(),
            LirOperation::ReadI32 { .. } => "int32_t".to_string(),
            LirOperation::ReadI64 { .. } => "int64_t".to_string(),
            LirOperation::ReadStruct { type_name, .. } => type_name.clone(),
            _ => "unknown".to_string(),
        }
    }

    fn generate_read_impl(&self, lir_type: &LirType, endianness: Endianness) -> Result<String> {
        let mut code = format!("inline {} {}::read(Reader& reader) {{\n", lir_type.name, lir_type.name);
        code.push_str(&format!("    {} result;\n", lir_type.name));

        let mut field_index = 0;
        for op in &lir_type.operations {
            if matches!(op, LirOperation::CreateStruct { .. }) {
                break;
            }

            code.push_str(&self.generate_read_operation(op, field_index, endianness)?);
            if !matches!(op, LirOperation::CreateStruct { .. } | LirOperation::AccessField { .. }) {
                field_index += 1;
            }
        }

        code.push_str("    return result;\n");
        code.push_str("}\n\n");

        Ok(code)
    }

    fn generate_read_operation(&self, op: &LirOperation, field_index: usize, endianness: Endianness) -> Result<String> {
        let field_name = format!("field_{}", field_index);
        let endian_suffix = match endianness {
            Endianness::Little => "_le",
            Endianness::Big => "_be",
            Endianness::Native => "_le",
        };

        Ok(match op {
            LirOperation::ReadU8 { .. } => {
                format!("    result.{} = reader.read_le<uint8_t>();\n", field_name)
            }
            LirOperation::ReadU16 { .. } => {
                format!("    result.{} = reader.read{}<uint16_t>();\n", field_name, endian_suffix)
            }
            LirOperation::ReadU32 { .. } => {
                format!("    result.{} = reader.read{}<uint32_t>();\n", field_name, endian_suffix)
            }
            LirOperation::ReadU64 { .. } => {
                format!("    result.{} = reader.read{}<uint64_t>();\n", field_name, endian_suffix)
            }
            LirOperation::ReadI8 { .. } => {
                format!("    result.{} = reader.read_le<int8_t>();\n", field_name)
            }
            LirOperation::ReadI16 { .. } => {
                format!("    result.{} = reader.read{}<int16_t>();\n", field_name, endian_suffix)
            }
            LirOperation::ReadI32 { .. } => {
                format!("    result.{} = reader.read{}<int32_t>();\n", field_name, endian_suffix)
            }
            LirOperation::ReadI64 { .. } => {
                format!("    result.{} = reader.read{}<int64_t>();\n", field_name, endian_suffix)
            }
            LirOperation::ReadArray { element_op, count, .. } => {
                let mut array_code = format!("    for (size_t i = 0; i < {}; ++i) {{\n", count);
                let element_read = self.generate_array_element_read(element_op, endianness)?;
                array_code.push_str(&format!("        result.{}[i] = {};\n", field_name, element_read));
                array_code.push_str("    }\n");
                array_code
            }
            LirOperation::ReadStruct { type_name, .. } => {
                format!("    result.{} = {}::read(reader);\n", field_name, type_name)
            }
            _ => String::new(),
        })
    }

    fn generate_array_element_read(&self, op: &LirOperation, endianness: Endianness) -> Result<String> {
        let endian_suffix = match endianness {
            Endianness::Little => "_le",
            Endianness::Big => "_be",
            Endianness::Native => "_le",
        };

        Ok(match op {
            LirOperation::ReadU8 { .. } => "reader.read_le<uint8_t>()".to_string(),
            LirOperation::ReadU16 { .. } => format!("reader.read{}<uint16_t>()", endian_suffix),
            LirOperation::ReadU32 { .. } => format!("reader.read{}<uint32_t>()", endian_suffix),
            LirOperation::ReadU64 { .. } => format!("reader.read{}<uint64_t>()", endian_suffix),
            LirOperation::ReadI8 { .. } => "reader.read_le<int8_t>()".to_string(),
            LirOperation::ReadI16 { .. } => format!("reader.read{}<int16_t>()", endian_suffix),
            LirOperation::ReadI32 { .. } => format!("reader.read{}<int32_t>()", endian_suffix),
            LirOperation::ReadI64 { .. } => format!("reader.read{}<int64_t>()", endian_suffix),
            LirOperation::ReadStruct { type_name, .. } => format!("{}::read(reader)", type_name),
            _ => "/* unsupported */".to_string(),
        })
    }

    fn generate_write_impl(&self, lir_type: &LirType, endianness: Endianness) -> Result<String> {
        let mut code = format!("inline void {}::write(Writer& writer) const {{\n", lir_type.name);

        let mut in_write_section = false;
        let mut field_index = 0;

        for op in &lir_type.operations {
            if matches!(op, LirOperation::AccessField { .. }) {
                in_write_section = true;
                continue;
            }

            if in_write_section {
                code.push_str(&self.generate_write_operation(op, field_index, endianness)?);
                if !matches!(op, LirOperation::AccessField { .. }) {
                    field_index += 1;
                }
            }
        }

        code.push_str("}\n\n");

        Ok(code)
    }

    fn generate_write_operation(&self, op: &LirOperation, field_index: usize, endianness: Endianness) -> Result<String> {
        let field_name = format!("field_{}", field_index);
        let endian_suffix = match endianness {
            Endianness::Little => "_le",
            Endianness::Big => "_be",
            Endianness::Native => "_le",
        };

        Ok(match op {
            LirOperation::WriteU8 { .. } => {
                format!("    writer.write_le({});\n", field_name)
            }
            LirOperation::WriteU16 { .. } => {
                format!("    writer.write{}({});\n", endian_suffix, field_name)
            }
            LirOperation::WriteU32 { .. } => {
                format!("    writer.write{}({});\n", endian_suffix, field_name)
            }
            LirOperation::WriteU64 { .. } => {
                format!("    writer.write{}({});\n", endian_suffix, field_name)
            }
            LirOperation::WriteI8 { .. } => {
                format!("    writer.write_le({});\n", field_name)
            }
            LirOperation::WriteI16 { .. } => {
                format!("    writer.write{}({});\n", endian_suffix, field_name)
            }
            LirOperation::WriteI32 { .. } => {
                format!("    writer.write{}({});\n", endian_suffix, field_name)
            }
            LirOperation::WriteI64 { .. } => {
                format!("    writer.write{}({});\n", endian_suffix, field_name)
            }
            LirOperation::WriteArray { element_op, count, .. } => {
                let mut array_code = format!("    for (size_t i = 0; i < {}; ++i) {{\n", count);
                let element_write = self.generate_array_element_write(element_op, &field_name, endianness)?;
                array_code.push_str(&format!("        {};\n", element_write));
                array_code.push_str("    }\n");
                array_code
            }
            LirOperation::WriteStruct { type_name: _, .. } => {
                format!("    {}.write(writer);\n", field_name)
            }
            _ => String::new(),
        })
    }

    fn generate_array_element_write(&self, op: &LirOperation, field_name: &str, endianness: Endianness) -> Result<String> {
        let endian_suffix = match endianness {
            Endianness::Little => "_le",
            Endianness::Big => "_be",
            Endianness::Native => "_le",
        };

        Ok(match op {
            LirOperation::WriteU8 { .. } => format!("writer.write_le({}[i])", field_name),
            LirOperation::WriteU16 { .. } => format!("writer.write{}({}[i])", endian_suffix, field_name),
            LirOperation::WriteU32 { .. } => format!("writer.write{}({}[i])", endian_suffix, field_name),
            LirOperation::WriteU64 { .. } => format!("writer.write{}({}[i])", endian_suffix, field_name),
            LirOperation::WriteI8 { .. } => format!("writer.write_le({}[i])", field_name),
            LirOperation::WriteI16 { .. } => format!("writer.write{}({}[i])", endian_suffix, field_name),
            LirOperation::WriteI32 { .. } => format!("writer.write{}({}[i])", endian_suffix, field_name),
            LirOperation::WriteI64 { .. } => format!("writer.write{}({}[i])", endian_suffix, field_name),
            LirOperation::WriteStruct { .. } => format!("{}[i].write(writer)", field_name),
            _ => "/* unsupported */".to_string(),
        })
    }
}

impl Backend for CppBackend {
    fn name(&self) -> &str {
        "cpp"
    }

    fn generate(&self, lir: &LirFormat) -> Result<GeneratedCode> {
        let namespace = lir.name.to_lowercase().replace('-', "_");
        let mut code = templates::generate_header_start(&namespace);

        let endianness = Endianness::Little;

        for lir_type in &lir.types {
            code.push_str(&self.generate_type(lir_type, endianness)?);
        }

        code.push_str(&templates::generate_header_end(&namespace));

        let filename = format!("{}.hpp", lir.name.to_lowercase().replace('-', "_"));

        Ok(GeneratedCode {
            files: vec![GeneratedFile {
                path: filename,
                content: code,
            }],
        })
    }
}

impl Default for CppBackend {
    fn default() -> Self {
        Self::new()
    }
}

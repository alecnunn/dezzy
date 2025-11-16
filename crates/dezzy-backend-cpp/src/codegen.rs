use crate::templates;
use anyhow::Result;
use dezzy_backend::{Backend, GeneratedCode, GeneratedFile};
use dezzy_core::hir::Endianness;
use dezzy_core::lir::{LirField, LirFormat, LirOperation, LirType, VarId};
use dezzy_core::topo_sort::topological_sort;
use std::collections::HashMap;

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
        let fields = lir_type
            .fields
            .iter()
            .map(|f| (f.name.clone(), self.lir_type_to_cpp_type(&f.type_info)))
            .collect();

        Ok(fields)
    }

    fn lir_type_to_cpp_type(&self, type_str: &str) -> String {
        if let Some(bracket_pos) = type_str.find('[') {
            if !type_str.ends_with(']') {
                return type_str.to_string();
            }
            let element_type = &type_str[..bracket_pos];
            let size_str = &type_str[bracket_pos + 1..type_str.len() - 1];
            let cpp_element_type = self.lir_type_to_cpp_type(element_type);

            // Try to parse as fixed size
            if let Ok(size) = size_str.parse::<usize>() {
                return format!("std::array<{}, {}>", cpp_element_type, size);
            } else {
                // Dynamic array (size from field)
                return format!("std::vector<{}>", cpp_element_type);
            }
        }

        match type_str {
            "u8" => "uint8_t".to_string(),
            "u16" => "uint16_t".to_string(),
            "u32" => "uint32_t".to_string(),
            "u64" => "uint64_t".to_string(),
            "i8" => "int8_t".to_string(),
            "i16" => "int16_t".to_string(),
            "i32" => "int32_t".to_string(),
            "i64" => "int64_t".to_string(),
            other => other.to_string(),
        }
    }


    fn generate_read_impl(&self, lir_type: &LirType, endianness: Endianness) -> Result<String> {
        let mut code = format!("inline {} {}::read(Reader& reader) {{\n", lir_type.name, lir_type.name);
        code.push_str(&format!("    {} result;\n", lir_type.name));

        let var_to_field = self.build_var_to_field_map(&lir_type.fields);

        for op in &lir_type.operations {
            if matches!(op, LirOperation::CreateStruct { .. }) {
                break;
            }

            code.push_str(&self.generate_read_operation(op, &var_to_field, endianness)?);
        }

        code.push_str("    return result;\n");
        code.push_str("}\n\n");

        Ok(code)
    }

    fn build_var_to_field_map(&self, fields: &[LirField]) -> HashMap<VarId, String> {
        fields
            .iter()
            .map(|f| (f.var_id, f.name.clone()))
            .collect()
    }

    fn generate_read_operation(
        &self,
        op: &LirOperation,
        var_to_field: &HashMap<VarId, String>,
        endianness: Endianness,
    ) -> Result<String> {
        let endian_suffix = match endianness {
            Endianness::Little => "_le",
            Endianness::Big => "_be",
            Endianness::Native => "_le",
        };

        Ok(match op {
            LirOperation::ReadU8 { dest } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                format!("    result.{} = reader.read_le<uint8_t>();\n", field_name)
            }
            LirOperation::ReadU16 { dest, .. } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                format!("    result.{} = reader.read{}<uint16_t>();\n", field_name, endian_suffix)
            }
            LirOperation::ReadU32 { dest, .. } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                format!("    result.{} = reader.read{}<uint32_t>();\n", field_name, endian_suffix)
            }
            LirOperation::ReadU64 { dest, .. } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                format!("    result.{} = reader.read{}<uint64_t>();\n", field_name, endian_suffix)
            }
            LirOperation::ReadI8 { dest } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                format!("    result.{} = reader.read_le<int8_t>();\n", field_name)
            }
            LirOperation::ReadI16 { dest, .. } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                format!("    result.{} = reader.read{}<int16_t>();\n", field_name, endian_suffix)
            }
            LirOperation::ReadI32 { dest, .. } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                format!("    result.{} = reader.read{}<int32_t>();\n", field_name, endian_suffix)
            }
            LirOperation::ReadI64 { dest, .. } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                format!("    result.{} = reader.read{}<int64_t>();\n", field_name, endian_suffix)
            }
            LirOperation::ReadArray { dest, element_op, count } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                let mut array_code = format!("    for (size_t i = 0; i < {}; ++i) {{\n", count);
                let element_read = self.generate_array_element_read(element_op, endianness)?;
                array_code.push_str(&format!("        result.{}[i] = {};\n", field_name, element_read));
                array_code.push_str("    }\n");
                array_code
            }
            LirOperation::ReadDynamicArray { dest, element_op, size_var } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                let size_field_name = var_to_field.get(size_var).map(|s| s.as_str()).unwrap_or("unknown_size");
                let mut array_code = format!("    result.{}.resize(result.{});\n", field_name, size_field_name);
                array_code.push_str(&format!("    for (size_t i = 0; i < result.{}; ++i) {{\n", size_field_name));
                let element_read = self.generate_array_element_read(element_op, endianness)?;
                array_code.push_str(&format!("        result.{}[i] = {};\n", field_name, element_read));
                array_code.push_str("    }\n");
                array_code
            }
            LirOperation::ReadStruct { dest, type_name } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
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

        let mut var_to_field: HashMap<VarId, String> = HashMap::new();
        let mut in_write_section = false;

        for op in &lir_type.operations {
            if let LirOperation::AccessField { dest, field_index, .. } = op {
                in_write_section = true;
                if *field_index < lir_type.fields.len() {
                    var_to_field.insert(*dest, lir_type.fields[*field_index].name.clone());
                }
                continue;
            }

            if in_write_section {
                code.push_str(&self.generate_write_operation(op, &var_to_field, endianness)?);
            }
        }

        code.push_str("}\n\n");

        Ok(code)
    }

    fn generate_write_operation(
        &self,
        op: &LirOperation,
        var_to_field: &HashMap<VarId, String>,
        endianness: Endianness,
    ) -> Result<String> {
        let endian_suffix = match endianness {
            Endianness::Little => "_le",
            Endianness::Big => "_be",
            Endianness::Native => "_le",
        };

        Ok(match op {
            LirOperation::WriteU8 { src } => {
                let field_name = var_to_field.get(src).map(|s| s.as_str()).unwrap_or("unknown");
                format!("    writer.write_le({});\n", field_name)
            }
            LirOperation::WriteU16 { src, .. } => {
                let field_name = var_to_field.get(src).map(|s| s.as_str()).unwrap_or("unknown");
                format!("    writer.write{}({});\n", endian_suffix, field_name)
            }
            LirOperation::WriteU32 { src, .. } => {
                let field_name = var_to_field.get(src).map(|s| s.as_str()).unwrap_or("unknown");
                format!("    writer.write{}({});\n", endian_suffix, field_name)
            }
            LirOperation::WriteU64 { src, .. } => {
                let field_name = var_to_field.get(src).map(|s| s.as_str()).unwrap_or("unknown");
                format!("    writer.write{}({});\n", endian_suffix, field_name)
            }
            LirOperation::WriteI8 { src } => {
                let field_name = var_to_field.get(src).map(|s| s.as_str()).unwrap_or("unknown");
                format!("    writer.write_le({});\n", field_name)
            }
            LirOperation::WriteI16 { src, .. } => {
                let field_name = var_to_field.get(src).map(|s| s.as_str()).unwrap_or("unknown");
                format!("    writer.write{}({});\n", endian_suffix, field_name)
            }
            LirOperation::WriteI32 { src, .. } => {
                let field_name = var_to_field.get(src).map(|s| s.as_str()).unwrap_or("unknown");
                format!("    writer.write{}({});\n", endian_suffix, field_name)
            }
            LirOperation::WriteI64 { src, .. } => {
                let field_name = var_to_field.get(src).map(|s| s.as_str()).unwrap_or("unknown");
                format!("    writer.write{}({});\n", endian_suffix, field_name)
            }
            LirOperation::WriteArray { src, element_op, count } => {
                let field_name = var_to_field.get(src).map(|s| s.as_str()).unwrap_or("unknown");
                let mut array_code = format!("    for (size_t i = 0; i < {}; ++i) {{\n", count);
                let element_write = self.generate_array_element_write(element_op, field_name, endianness)?;
                array_code.push_str(&format!("        {};\n", element_write));
                array_code.push_str("    }\n");
                array_code
            }
            LirOperation::WriteDynamicArray { src, element_op, size_field_name, .. } => {
                let field_name = var_to_field.get(src).map(|s| s.as_str()).unwrap_or("unknown");
                let mut array_code = format!("    for (size_t i = 0; i < {}; ++i) {{\n", size_field_name);
                let element_write = self.generate_array_element_write(element_op, field_name, endianness)?;
                array_code.push_str(&format!("        {};\n", element_write));
                array_code.push_str("    }\n");
                array_code
            }
            LirOperation::WriteStruct { src, .. } => {
                let field_name = var_to_field.get(src).map(|s| s.as_str()).unwrap_or("unknown");
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
        let mut lir_sorted = lir.clone();
        topological_sort(&mut lir_sorted)?;

        let namespace = lir_sorted.name.to_lowercase().replace('-', "_");
        let mut code = templates::generate_header_start(&namespace);

        for lir_type in &lir_sorted.types {
            code.push_str(&self.generate_type(lir_type, lir_sorted.endianness)?);
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

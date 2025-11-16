use crate::expr_codegen::generate_expr;
use crate::templates;
use anyhow::Result;
use dezzy_backend::{Backend, GeneratedCode, GeneratedFile};
use dezzy_core::hir::{Endianness, HirAssertion, HirAssertValue, HirEnum, HirPrimitiveType};
use dezzy_core::lir::{LirField, LirFormat, LirOperation, LirType, VarId};
use dezzy_core::topo_sort::topological_sort;
use std::collections::HashMap;

pub struct CppBackend;

impl CppBackend {
    pub fn new() -> Self {
        Self
    }

    fn generate_enum(&self, enum_def: &HirEnum) -> String {
        let underlying_type = match enum_def.underlying_type {
            HirPrimitiveType::U8 => "uint8_t",
            HirPrimitiveType::U16 => "uint16_t",
            HirPrimitiveType::U32 => "uint32_t",
            HirPrimitiveType::U64 => "uint64_t",
            HirPrimitiveType::I8 => "int8_t",
            HirPrimitiveType::I16 => "int16_t",
            HirPrimitiveType::I32 => "int32_t",
            HirPrimitiveType::I64 => "int64_t",
        };

        let mut code = format!("enum class {} : {} {{\n", enum_def.name, underlying_type);

        for (i, value) in enum_def.values.iter().enumerate() {
            if i > 0 {
                code.push_str(",\n");
            }
            code.push_str(&format!("    {} = {}", value.name, value.value));
        }

        code.push_str("\n};\n\n");
        code
    }

    fn generate_type(&self, lir_type: &LirType, endianness: Endianness, enums: &[HirEnum]) -> Result<String> {
        let fields = self.extract_fields(lir_type)?;
        let mut code = templates::generate_struct_declaration(&lir_type.name, &fields);

        code.push_str(&self.generate_read_impl(lir_type, endianness, enums)?);
        code.push_str(&self.generate_write_impl(lir_type, endianness, enums)?);

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
        // Handle string types
        if type_str == "cstr" || type_str.starts_with("str(") {
            return "std::string".to_string();
        }

        if let Some(bracket_pos) = type_str.find('[') {
            if !type_str.ends_with(']') {
                return type_str.to_string();
            }
            let element_type = &type_str[..bracket_pos];
            let size_str = &type_str[bracket_pos + 1..type_str.len() - 1];

            // Special case: str[N] is a fixed-length string
            if element_type == "str" {
                return "std::string".to_string();
            }

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


    fn generate_read_impl(&self, lir_type: &LirType, endianness: Endianness, enums: &[HirEnum]) -> Result<String> {
        let mut code = format!("inline {} {}::read(Reader& reader) {{\n", lir_type.name, lir_type.name);
        code.push_str(&format!("    {} result;\n", lir_type.name));

        let var_to_field = self.build_var_to_field_map(&lir_type.fields);

        // Build enum map: enum_name -> underlying_type
        let mut enum_types = HashMap::new();
        for enum_def in enums {
            enum_types.insert(enum_def.name.clone(), enum_def.underlying_type);
        }

        for op in &lir_type.operations {
            if matches!(op, LirOperation::CreateStruct { .. }) {
                break;
            }

            code.push_str(&self.generate_read_operation(op, &var_to_field, &lir_type.fields, &enum_types, endianness)?);
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

    fn generate_assertion_check(
        &self,
        field_name: &str,
        assertion: &HirAssertion,
    ) -> String {
        let mut code = String::new();

        match assertion {
            HirAssertion::Equals(assert_val) => {
                match assert_val {
                    HirAssertValue::Int(value) => {
                        code.push_str(&format!("    if (result.{} != {}) {{\n", field_name, value));
                        code.push_str(&format!("        throw ParseError(\"Field '{}' must equal {}, got \" + std::to_string(result.{}));\n", field_name, value, field_name));
                        code.push_str("    }\n");
                    }
                    HirAssertValue::IntArray(values) => {
                        code.push_str("    {\n");
                        code.push_str(&format!("        std::array<uint8_t, {}> expected = {{", values.len()));
                        for (i, val) in values.iter().enumerate() {
                            if i > 0 { code.push_str(", "); }
                            code.push_str(&format!("{}", val));
                        }
                        code.push_str("};\n");
                        code.push_str(&format!("        if (!std::equal(result.{}.begin(), result.{}.end(), expected.begin())) {{\n", field_name, field_name));
                        code.push_str(&format!("            throw ParseError(\"Field '{}' does not match expected value\");\n", field_name));
                        code.push_str("        }\n");
                        code.push_str("    }\n");
                    }
                }
            }
            HirAssertion::NotEquals(assert_val) => {
                match assert_val {
                    HirAssertValue::Int(value) => {
                        code.push_str(&format!("    if (result.{} == {}) {{\n", field_name, value));
                        code.push_str(&format!("        throw ParseError(\"Field '{}' must not equal {}\");\n", field_name, value));
                        code.push_str("    }\n");
                    }
                    HirAssertValue::IntArray(_) => {
                        code.push_str(&format!("    // NotEquals array assertion not implemented for field '{}'\n", field_name));
                    }
                }
            }
            HirAssertion::GreaterThan(threshold) => {
                code.push_str(&format!("    if (result.{} <= {}) {{\n", field_name, threshold));
                code.push_str(&format!("        throw ParseError(\"Field '{}' must be greater than {}, got \" + std::to_string(result.{}));\n", field_name, threshold, field_name));
                code.push_str("    }\n");
            }
            HirAssertion::GreaterOrEqual(threshold) => {
                code.push_str(&format!("    if (result.{} < {}) {{\n", field_name, threshold));
                code.push_str(&format!("        throw ParseError(\"Field '{}' must be >= {}, got \" + std::to_string(result.{}));\n", field_name, threshold, field_name));
                code.push_str("    }\n");
            }
            HirAssertion::LessThan(threshold) => {
                code.push_str(&format!("    if (result.{} >= {}) {{\n", field_name, threshold));
                code.push_str(&format!("        throw ParseError(\"Field '{}' must be less than {}, got \" + std::to_string(result.{}));\n", field_name, threshold, field_name));
                code.push_str("    }\n");
            }
            HirAssertion::LessOrEqual(threshold) => {
                code.push_str(&format!("    if (result.{} > {}) {{\n", field_name, threshold));
                code.push_str(&format!("        throw ParseError(\"Field '{}' must be <= {}, got \" + std::to_string(result.{}));\n", field_name, threshold, field_name));
                code.push_str("    }\n");
            }
            HirAssertion::In(values) => {
                code.push_str("    {\n");
                code.push_str(&format!("        std::array<int64_t, {}> allowed = {{", values.len()));
                for (i, val) in values.iter().enumerate() {
                    if i > 0 { code.push_str(", "); }
                    code.push_str(&format!("{}", val));
                }
                code.push_str("};\n");
                code.push_str(&format!("        if (std::find(allowed.begin(), allowed.end(), result.{}) == allowed.end()) {{\n", field_name));
                code.push_str(&format!("            throw ParseError(\"Field '{}' has invalid value \" + std::to_string(result.{}));\n", field_name, field_name));
                code.push_str("        }\n");
                code.push_str("    }\n");
            }
            HirAssertion::NotIn(values) => {
                code.push_str("    {\n");
                code.push_str(&format!("        std::array<int64_t, {}> forbidden = {{", values.len()));
                for (i, val) in values.iter().enumerate() {
                    if i > 0 { code.push_str(", "); }
                    code.push_str(&format!("{}", val));
                }
                code.push_str("};\n");
                code.push_str(&format!("        if (std::find(forbidden.begin(), forbidden.end(), result.{}) != forbidden.end()) {{\n", field_name));
                code.push_str(&format!("            throw ParseError(\"Field '{}' has forbidden value \" + std::to_string(result.{}));\n", field_name, field_name));
                code.push_str("        }\n");
                code.push_str("    }\n");
            }
            HirAssertion::Range { min, max } => {
                code.push_str(&format!("    if (result.{} < {} || result.{} > {}) {{\n", field_name, min, field_name, max));
                code.push_str(&format!("        throw ParseError(\"Field '{}' must be in range [{}, {}], got \" + std::to_string(result.{}));\n", field_name, min, max, field_name));
                code.push_str("    }\n");
            }
        }

        code
    }

    fn generate_read_operation(
        &self,
        op: &LirOperation,
        var_to_field: &HashMap<VarId, String>,
        fields: &[LirField],
        enum_types: &HashMap<String, HirPrimitiveType>,
        endianness: Endianness,
    ) -> Result<String> {
        let endian_suffix = match endianness {
            Endianness::Little => "_le",
            Endianness::Big => "_be",
            Endianness::Native => "_le",
        };

        // Helper to find if a field is an enum and return its name
        let get_enum_type_for_dest = |dest: &VarId| -> Option<String> {
            let field_name = var_to_field.get(dest)?;
            let field = fields.iter().find(|f| f.var_id == *dest)?;
            if enum_types.contains_key(&field.type_info) {
                Some(field.type_info.clone())
            } else {
                None
            }
        };

        // Helper to add assertion check if field has one
        let add_assertion = |code: &mut String, dest: &VarId| {
            if let Some(field) = fields.iter().find(|f| f.var_id == *dest) {
                if let Some(ref assertion) = field.assertion {
                    code.push_str(&self.generate_assertion_check(&field.name, assertion));
                }
            }
        };

        Ok(match op {
            LirOperation::ReadU8 { dest } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                let mut code = if let Some(enum_name) = get_enum_type_for_dest(dest) {
                    format!("    result.{} = static_cast<{}>(reader.read_le<uint8_t>());\n", field_name, enum_name)
                } else {
                    format!("    result.{} = reader.read_le<uint8_t>();\n", field_name)
                };
                add_assertion(&mut code, dest);
                code
            }
            LirOperation::ReadU16 { dest, .. } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                let mut code = if let Some(enum_name) = get_enum_type_for_dest(dest) {
                    format!("    result.{} = static_cast<{}>(reader.read{}<uint16_t>());\n", field_name, enum_name, endian_suffix)
                } else {
                    format!("    result.{} = reader.read{}<uint16_t>();\n", field_name, endian_suffix)
                };
                add_assertion(&mut code, dest);
                code
            }
            LirOperation::ReadU32 { dest, .. } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                let mut code = if let Some(enum_name) = get_enum_type_for_dest(dest) {
                    format!("    result.{} = static_cast<{}>(reader.read{}<uint32_t>());\n", field_name, enum_name, endian_suffix)
                } else {
                    format!("    result.{} = reader.read{}<uint32_t>();\n", field_name, endian_suffix)
                };
                add_assertion(&mut code, dest);
                code
            }
            LirOperation::ReadU64 { dest, .. } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                let mut code = if let Some(enum_name) = get_enum_type_for_dest(dest) {
                    format!("    result.{} = static_cast<{}>(reader.read{}<uint64_t>());\n", field_name, enum_name, endian_suffix)
                } else {
                    format!("    result.{} = reader.read{}<uint64_t>();\n", field_name, endian_suffix)
                };
                add_assertion(&mut code, dest);
                code
            }
            LirOperation::ReadI8 { dest } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                let mut code = if let Some(enum_name) = get_enum_type_for_dest(dest) {
                    format!("    result.{} = static_cast<{}>(reader.read_le<int8_t>());\n", field_name, enum_name)
                } else {
                    format!("    result.{} = reader.read_le<int8_t>();\n", field_name)
                };
                add_assertion(&mut code, dest);
                code
            }
            LirOperation::ReadI16 { dest, .. } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                let mut code = if let Some(enum_name) = get_enum_type_for_dest(dest) {
                    format!("    result.{} = static_cast<{}>(reader.read{}<int16_t>());\n", field_name, enum_name, endian_suffix)
                } else {
                    format!("    result.{} = reader.read{}<int16_t>();\n", field_name, endian_suffix)
                };
                add_assertion(&mut code, dest);
                code
            }
            LirOperation::ReadI32 { dest, .. } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                let mut code = if let Some(enum_name) = get_enum_type_for_dest(dest) {
                    format!("    result.{} = static_cast<{}>(reader.read{}<int32_t>());\n", field_name, enum_name, endian_suffix)
                } else {
                    format!("    result.{} = reader.read{}<int32_t>();\n", field_name, endian_suffix)
                };
                add_assertion(&mut code, dest);
                code
            }
            LirOperation::ReadI64 { dest, .. } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                let mut code = if let Some(enum_name) = get_enum_type_for_dest(dest) {
                    format!("    result.{} = static_cast<{}>(reader.read{}<int64_t>());\n", field_name, enum_name, endian_suffix)
                } else {
                    format!("    result.{} = reader.read{}<int64_t>();\n", field_name, endian_suffix)
                };
                add_assertion(&mut code, dest);
                code
            }
            LirOperation::ReadArray { dest, element_op, count } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                let mut array_code = format!("    for (size_t i = 0; i < {}; ++i) {{\n", count);
                let element_read = self.generate_array_element_read(element_op, endianness)?;
                array_code.push_str(&format!("        result.{}[i] = {};\n", field_name, element_read));
                array_code.push_str("    }\n");
                add_assertion(&mut array_code, dest);
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
            LirOperation::ReadUntilEofArray { dest, element_op } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                let mut array_code = String::from("    while (reader.remaining() > 0) {\n");
                let element_read = self.generate_array_element_read(element_op, endianness)?;
                array_code.push_str(&format!("        result.{}.push_back({});\n", field_name, element_read));
                array_code.push_str("    }\n");
                array_code
            }
            LirOperation::ReadUntilConditionArray { dest, element_op, condition } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                let mut array_code = String::from("    do {\n");
                let element_read = self.generate_array_element_read(element_op, endianness)?;
                array_code.push_str(&format!("        result.{}.push_back({});\n", field_name, element_read));
                array_code.push_str("    } while (");
                // Generate condition - negated because we continue while condition is false
                let condition_code = generate_expr(condition, &format!("result.{}", field_name))?;
                array_code.push_str(&format!("!{}", condition_code));
                array_code.push_str(");\n");
                array_code
            }
            LirOperation::ReadStruct { dest, type_name } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                format!("    result.{} = {}::read(reader);\n", field_name, type_name)
            }
            LirOperation::ReadFixedString { dest, length } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                let mut code = String::new();
                code.push_str("    {\n");
                code.push_str(&format!("        std::vector<uint8_t> bytes({});\n", length));
                code.push_str(&format!("        for (size_t i = 0; i < {}; ++i) {{\n", length));
                code.push_str("            bytes[i] = reader.read_le<uint8_t>();\n");
                code.push_str("        }\n");
                code.push_str(&format!("        result.{} = std::string(reinterpret_cast<const char*>(bytes.data()), bytes.size());\n", field_name));
                code.push_str("    }\n");
                add_assertion(&mut code, dest);
                code
            }
            LirOperation::ReadNullTerminatedString { dest } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                let mut code = String::new();
                code.push_str("    {\n");
                code.push_str("        std::vector<uint8_t> bytes;\n");
                code.push_str("        uint8_t byte;\n");
                code.push_str("        while ((byte = reader.read_le<uint8_t>()) != 0) {\n");
                code.push_str("            bytes.push_back(byte);\n");
                code.push_str("        }\n");
                code.push_str(&format!("        result.{} = std::string(reinterpret_cast<const char*>(bytes.data()), bytes.size());\n", field_name));
                code.push_str("    }\n");
                add_assertion(&mut code, dest);
                code
            }
            LirOperation::ReadLengthPrefixedString { dest, length_var } => {
                let field_name = var_to_field.get(dest).map(|s| s.as_str()).unwrap_or("unknown");
                let length_field = var_to_field.get(length_var).map(|s| s.as_str()).unwrap_or("unknown");
                let mut code = String::new();
                code.push_str("    {\n");
                code.push_str(&format!("        std::vector<uint8_t> bytes(result.{});\n", length_field));
                code.push_str(&format!("        for (size_t i = 0; i < result.{}; ++i) {{\n", length_field));
                code.push_str("            bytes[i] = reader.read_le<uint8_t>();\n");
                code.push_str("        }\n");
                code.push_str(&format!("        result.{} = std::string(reinterpret_cast<const char*>(bytes.data()), bytes.size());\n", field_name));
                code.push_str("    }\n");
                add_assertion(&mut code, dest);
                code
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

    fn generate_write_impl(&self, lir_type: &LirType, endianness: Endianness, enums: &[HirEnum]) -> Result<String> {
        let mut code = format!("inline void {}::write(Writer& writer) const {{\n", lir_type.name);

        let mut var_to_field: HashMap<VarId, String> = HashMap::new();
        let mut in_write_section = false;

        // Build enum map: enum_name -> underlying_type
        let mut enum_types = HashMap::new();
        for enum_def in enums {
            enum_types.insert(enum_def.name.clone(), enum_def.underlying_type);
        }

        for op in &lir_type.operations {
            if let LirOperation::AccessField { dest, field_index, .. } = op {
                in_write_section = true;
                if *field_index < lir_type.fields.len() {
                    var_to_field.insert(*dest, lir_type.fields[*field_index].name.clone());
                }
                continue;
            }

            if in_write_section {
                code.push_str(&self.generate_write_operation(op, &var_to_field, &lir_type.fields, &enum_types, endianness)?);
            }
        }

        code.push_str("}\n\n");

        Ok(code)
    }

    fn generate_write_operation(
        &self,
        op: &LirOperation,
        var_to_field: &HashMap<VarId, String>,
        fields: &[LirField],
        enum_types: &HashMap<String, HirPrimitiveType>,
        endianness: Endianness,
    ) -> Result<String> {
        let endian_suffix = match endianness {
            Endianness::Little => "_le",
            Endianness::Big => "_be",
            Endianness::Native => "_le",
        };

        // Helper to find if a field is an enum and return cast string
        let get_field_with_cast = |src: &VarId, cpp_type: &str| -> String {
            let field_name = var_to_field.get(src).map(|s| s.as_str()).unwrap_or("unknown");
            if let Some(field) = fields.iter().find(|f| var_to_field.get(&f.var_id).map(|s| s.as_str()) == Some(field_name)) {
                if enum_types.contains_key(&field.type_info) {
                    return format!("static_cast<{}>({})", cpp_type, field_name);
                }
            }
            field_name.to_string()
        };

        Ok(match op {
            LirOperation::WriteU8 { src } => {
                let value_expr = get_field_with_cast(src, "uint8_t");
                format!("    writer.write_le({});\n", value_expr)
            }
            LirOperation::WriteU16 { src, .. } => {
                let value_expr = get_field_with_cast(src, "uint16_t");
                format!("    writer.write{}({});\n", endian_suffix, value_expr)
            }
            LirOperation::WriteU32 { src, .. } => {
                let value_expr = get_field_with_cast(src, "uint32_t");
                format!("    writer.write{}({});\n", endian_suffix, value_expr)
            }
            LirOperation::WriteU64 { src, .. } => {
                let value_expr = get_field_with_cast(src, "uint64_t");
                format!("    writer.write{}({});\n", endian_suffix, value_expr)
            }
            LirOperation::WriteI8 { src } => {
                let value_expr = get_field_with_cast(src, "int8_t");
                format!("    writer.write_le({});\n", value_expr)
            }
            LirOperation::WriteI16 { src, .. } => {
                let value_expr = get_field_with_cast(src, "int16_t");
                format!("    writer.write{}({});\n", endian_suffix, value_expr)
            }
            LirOperation::WriteI32 { src, .. } => {
                let value_expr = get_field_with_cast(src, "int32_t");
                format!("    writer.write{}({});\n", endian_suffix, value_expr)
            }
            LirOperation::WriteI64 { src, .. } => {
                let value_expr = get_field_with_cast(src, "int64_t");
                format!("    writer.write{}({});\n", endian_suffix, value_expr)
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
            LirOperation::WriteUntilEofArray { src, element_op } => {
                let field_name = var_to_field.get(src).map(|s| s.as_str()).unwrap_or("unknown");
                let mut array_code = format!("    for (size_t i = 0; i < {}.size(); ++i) {{\n", field_name);
                let element_write = self.generate_array_element_write(element_op, field_name, endianness)?;
                array_code.push_str(&format!("        {};\n", element_write));
                array_code.push_str("    }\n");
                array_code
            }
            LirOperation::WriteUntilConditionArray { src, element_op } => {
                // Same as WriteUntilEofArray - just write all elements in the array
                let field_name = var_to_field.get(src).map(|s| s.as_str()).unwrap_or("unknown");
                let mut array_code = format!("    for (size_t i = 0; i < {}.size(); ++i) {{\n", field_name);
                let element_write = self.generate_array_element_write(element_op, field_name, endianness)?;
                array_code.push_str(&format!("        {};\n", element_write));
                array_code.push_str("    }\n");
                array_code
            }
            LirOperation::WriteStruct { src, .. } => {
                let field_name = var_to_field.get(src).map(|s| s.as_str()).unwrap_or("unknown");
                format!("    {}.write(writer);\n", field_name)
            }
            LirOperation::WriteFixedString { src, length } => {
                let field_name = var_to_field.get(src).map(|s| s.as_str()).unwrap_or("unknown");
                let mut code = String::new();
                code.push_str(&format!("    for (size_t i = 0; i < {}; ++i) {{\n", length));
                code.push_str(&format!("        writer.write_le(static_cast<uint8_t>({}.size() > i ? {}[i] : 0));\n", field_name, field_name));
                code.push_str("    }\n");
                code
            }
            LirOperation::WriteNullTerminatedString { src } => {
                let field_name = var_to_field.get(src).map(|s| s.as_str()).unwrap_or("unknown");
                let mut code = String::new();
                code.push_str(&format!("    for (size_t i = 0; i < {}.size(); ++i) {{\n", field_name));
                code.push_str(&format!("        writer.write_le(static_cast<uint8_t>({}[i]));\n", field_name));
                code.push_str("    }\n");
                code.push_str("    writer.write_le(static_cast<uint8_t>(0));  // null terminator\n");
                code
            }
            LirOperation::WriteLengthPrefixedString { src, .. } => {
                let field_name = var_to_field.get(src).map(|s| s.as_str()).unwrap_or("unknown");
                let mut code = String::new();
                code.push_str(&format!("    for (size_t i = 0; i < {}.size(); ++i) {{\n", field_name));
                code.push_str(&format!("        writer.write_le(static_cast<uint8_t>({}[i]));\n", field_name));
                code.push_str("    }\n");
                code
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

        // Generate enum definitions first
        for enum_def in &lir_sorted.enums {
            code.push_str(&self.generate_enum(enum_def));
        }

        for lir_type in &lir_sorted.types {
            code.push_str(&self.generate_type(lir_type, lir_sorted.endianness, &lir_sorted.enums)?);
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

use crate::hir::{HirFormat, HirPrimitiveType, HirStruct, HirType, HirTypeDef};
use crate::lir::{LirField, LirFormat, LirOperation, LirType, VarId};
use std::collections::HashMap;
use thiserror::Error;

fn primitive_to_hir_type(prim: HirPrimitiveType) -> HirType {
    match prim {
        HirPrimitiveType::U8 => HirType::U8,
        HirPrimitiveType::U16 => HirType::U16,
        HirPrimitiveType::U32 => HirType::U32,
        HirPrimitiveType::U64 => HirType::U64,
        HirPrimitiveType::I8 => HirType::I8,
        HirPrimitiveType::I16 => HirType::I16,
        HirPrimitiveType::I32 => HirType::I32,
        HirPrimitiveType::I64 => HirType::I64,
        HirPrimitiveType::U1 => HirType::U1,
        HirPrimitiveType::U2 => HirType::U2,
        HirPrimitiveType::U3 => HirType::U3,
        HirPrimitiveType::U4 => HirType::U4,
        HirPrimitiveType::U5 => HirType::U5,
        HirPrimitiveType::U6 => HirType::U6,
        HirPrimitiveType::U7 => HirType::U7,
        HirPrimitiveType::I1 => HirType::I1,
        HirPrimitiveType::I2 => HirType::I2,
        HirPrimitiveType::I3 => HirType::I3,
        HirPrimitiveType::I4 => HirType::I4,
        HirPrimitiveType::I5 => HirType::I5,
        HirPrimitiveType::I6 => HirType::I6,
        HirPrimitiveType::I7 => HirType::I7,
    }
}

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Unknown type reference: {0}")]
    UnknownType(String),
    #[error("Recursive type reference: {0}")]
    RecursiveType(String),
}

pub struct Pipeline {
    next_var_id: VarId,
}

impl Pipeline {
    pub fn new() -> Self {
        Self { next_var_id: 0 }
    }

    pub fn lower(&mut self, hir: HirFormat) -> Result<LirFormat, PipelineError> {
        let mut lir_types = Vec::new();

        for type_def in &hir.types {
            match type_def {
                HirTypeDef::Struct(struct_def) => {
                    let lir_type = self.lower_struct(struct_def, &hir)?;
                    lir_types.push(lir_type);
                }
            }
        }

        Ok(LirFormat {
            name: hir.name,
            enums: hir.enums,
            types: lir_types,
            endianness: hir.endianness,
        })
    }

    fn lower_struct(
        &mut self,
        struct_def: &HirStruct,
        format: &HirFormat,
    ) -> Result<LirType, PipelineError> {
        let mut read_ops = Vec::new();
        let mut field_vars = Vec::new();
        let mut lir_fields = Vec::new();
        let mut field_name_to_var: HashMap<String, VarId> = HashMap::new();

        for field in &struct_def.fields {
            let field_var = self.next_var();
            field_vars.push(field_var);
            field_name_to_var.insert(field.name.clone(), field_var);

            let type_info = self.hir_type_to_string(&field.field_type);

            // Convert Skip enum to Option<String> for LIR (just for metadata)
            let skip_marker = field.skip.as_ref().map(|_| "skip".to_string());

            lir_fields.push(LirField {
                name: field.name.clone(),
                doc: field.doc.clone(),
                var_id: field_var,
                type_info,
                assertion: field.assertion.clone(),
                skip: skip_marker,
            });

            // If this is a skip/pad/align field, generate appropriate operation instead of read
            if let Some(ref skip) = field.skip {
                use crate::hir::Skip;
                match skip {
                    Skip::Variable(size_field) => {
                        let size_var = *field_name_to_var.get(size_field).ok_or_else(|| {
                            PipelineError::UnknownType(format!("Skip size field '{}' not found", size_field))
                        })?;
                        read_ops.push(LirOperation::Skip { size_var });
                    }
                    Skip::Fixed(bytes) => {
                        read_ops.push(LirOperation::PadFixed { bytes: *bytes });
                    }
                    Skip::Align(boundary) => {
                        read_ops.push(LirOperation::Align { boundary: *boundary });
                    }
                }
            } else {
                let read_op = self.lower_read_type(&field.field_type, field_var, format, &field_name_to_var)?;
                read_ops.push(read_op);
            }
        }

        let result_var = self.next_var();
        read_ops.push(LirOperation::CreateStruct {
            dest: result_var,
            type_name: struct_def.name.clone(),
            fields: field_vars.clone(),
        });

        let mut write_ops = Vec::new();
        let write_param = self.next_var();

        for (idx, field) in struct_def.fields.iter().enumerate() {
            // Skip fields with skip directive - they should not be written
            if field.skip.is_some() {
                continue;
            }

            let field_var = self.next_var();
            write_ops.push(LirOperation::AccessField {
                dest: field_var,
                struct_var: write_param,
                field_index: idx,
            });

            let write_op = self.lower_write_type(&field.field_type, field_var, format, &field_name_to_var)?;
            write_ops.push(write_op);
        }

        let mut all_ops = read_ops;
        all_ops.extend(write_ops);

        Ok(LirType {
            name: struct_def.name.clone(),
            fields: lir_fields,
            operations: all_ops,
            read_result: result_var,
            write_param,
        })
    }

    fn hir_type_to_string(&self, ty: &HirType) -> String {
        match ty {
            HirType::U8 => "u8".to_string(),
            HirType::U16 => "u16".to_string(),
            HirType::U32 => "u32".to_string(),
            HirType::U64 => "u64".to_string(),
            HirType::I8 => "i8".to_string(),
            HirType::I16 => "i16".to_string(),
            HirType::I32 => "i32".to_string(),
            HirType::I64 => "i64".to_string(),
            HirType::U1 => "u1".to_string(),
            HirType::U2 => "u2".to_string(),
            HirType::U3 => "u3".to_string(),
            HirType::U4 => "u4".to_string(),
            HirType::U5 => "u5".to_string(),
            HirType::U6 => "u6".to_string(),
            HirType::U7 => "u7".to_string(),
            HirType::I1 => "i1".to_string(),
            HirType::I2 => "i2".to_string(),
            HirType::I3 => "i3".to_string(),
            HirType::I4 => "i4".to_string(),
            HirType::I5 => "i5".to_string(),
            HirType::I6 => "i6".to_string(),
            HirType::I7 => "i7".to_string(),
            HirType::Array { element_type, size } => {
                format!("{}[{}]", self.hir_type_to_string(element_type), size)
            }
            HirType::DynamicArray { element_type, size_field } => {
                format!("{}[{}]", self.hir_type_to_string(element_type), size_field)
            }
            HirType::UntilEofArray { element_type } => {
                format!("{}[]", self.hir_type_to_string(element_type))
            }
            HirType::UntilConditionArray { element_type, .. } => {
                format!("{}[]", self.hir_type_to_string(element_type))
            }
            HirType::FixedString { size } => format!("str[{}]", size),
            HirType::NullTerminatedString => "cstr".to_string(),
            HirType::LengthPrefixedString { length_field } => format!("str({})", length_field),
            HirType::Blob { size_field } => format!("blob({})", size_field),
            HirType::Enum(name) => name.clone(),
            HirType::UserDefined(name) => name.clone(),
        }
    }

    fn lower_read_type(
        &mut self,
        ty: &HirType,
        dest: VarId,
        format: &HirFormat,
        field_map: &HashMap<String, VarId>,
    ) -> Result<LirOperation, PipelineError> {
        Ok(match ty {
            HirType::U8 => LirOperation::ReadU8 { dest },
            HirType::U16 => LirOperation::ReadU16 {
                dest,
                endianness: format.endianness,
            },
            HirType::U32 => LirOperation::ReadU32 {
                dest,
                endianness: format.endianness,
            },
            HirType::U64 => LirOperation::ReadU64 {
                dest,
                endianness: format.endianness,
            },
            HirType::I8 => LirOperation::ReadI8 { dest },
            HirType::I16 => LirOperation::ReadI16 {
                dest,
                endianness: format.endianness,
            },
            HirType::I32 => LirOperation::ReadI32 {
                dest,
                endianness: format.endianness,
            },
            HirType::I64 => LirOperation::ReadI64 {
                dest,
                endianness: format.endianness,
            },
            // Bitfield types
            HirType::U1 => LirOperation::ReadBits { dest, num_bits: 1, signed: false },
            HirType::U2 => LirOperation::ReadBits { dest, num_bits: 2, signed: false },
            HirType::U3 => LirOperation::ReadBits { dest, num_bits: 3, signed: false },
            HirType::U4 => LirOperation::ReadBits { dest, num_bits: 4, signed: false },
            HirType::U5 => LirOperation::ReadBits { dest, num_bits: 5, signed: false },
            HirType::U6 => LirOperation::ReadBits { dest, num_bits: 6, signed: false },
            HirType::U7 => LirOperation::ReadBits { dest, num_bits: 7, signed: false },
            HirType::I1 => LirOperation::ReadBits { dest, num_bits: 1, signed: true },
            HirType::I2 => LirOperation::ReadBits { dest, num_bits: 2, signed: true },
            HirType::I3 => LirOperation::ReadBits { dest, num_bits: 3, signed: true },
            HirType::I4 => LirOperation::ReadBits { dest, num_bits: 4, signed: true },
            HirType::I5 => LirOperation::ReadBits { dest, num_bits: 5, signed: true },
            HirType::I6 => LirOperation::ReadBits { dest, num_bits: 6, signed: true },
            HirType::I7 => LirOperation::ReadBits { dest, num_bits: 7, signed: true },
            HirType::Array { element_type, size } => {
                let dummy_var = self.next_var();
                let element_op = self.lower_read_type(element_type, dummy_var, format, field_map)?;
                LirOperation::ReadArray {
                    dest,
                    element_op: Box::new(element_op),
                    count: *size,
                }
            }
            HirType::DynamicArray { element_type, size_field } => {
                let size_var = *field_map.get(size_field).ok_or_else(|| {
                    PipelineError::UnknownType(format!("Size field '{}' not found", size_field))
                })?;
                let dummy_var = self.next_var();
                let element_op = self.lower_read_type(element_type, dummy_var, format, field_map)?;
                LirOperation::ReadDynamicArray {
                    dest,
                    element_op: Box::new(element_op),
                    size_var,
                }
            }
            HirType::UntilEofArray { element_type } => {
                let dummy_var = self.next_var();
                let element_op = self.lower_read_type(element_type, dummy_var, format, field_map)?;
                LirOperation::ReadUntilEofArray {
                    dest,
                    element_op: Box::new(element_op),
                }
            }
            HirType::UntilConditionArray { element_type, condition } => {
                let dummy_var = self.next_var();
                let element_op = self.lower_read_type(element_type, dummy_var, format, field_map)?;
                LirOperation::ReadUntilConditionArray {
                    dest,
                    element_op: Box::new(element_op),
                    condition: condition.clone(),
                }
            }
            HirType::FixedString { size } => LirOperation::ReadFixedString {
                dest,
                length: *size,
            },
            HirType::NullTerminatedString => LirOperation::ReadNullTerminatedString { dest },
            HirType::LengthPrefixedString { length_field } => {
                let length_var = *field_map.get(length_field).ok_or_else(|| {
                    PipelineError::UnknownType(format!("Length field '{}' not found", length_field))
                })?;
                LirOperation::ReadLengthPrefixedString {
                    dest,
                    length_var,
                }
            }
            HirType::Blob { size_field } => {
                let size_var = *field_map.get(size_field).ok_or_else(|| {
                    PipelineError::UnknownType(format!("Blob size field '{}' not found", size_field))
                })?;
                LirOperation::ReadBlob {
                    dest,
                    size_var,
                }
            }
            HirType::Enum(name) => {
                // Look up the enum definition
                let enum_def = format.enums.iter().find(|e| &e.name == name).ok_or_else(|| {
                    PipelineError::UnknownType(format!("Enum '{}' not found", name))
                })?;

                // Lower as the underlying primitive type
                let underlying_hir_type = primitive_to_hir_type(enum_def.underlying_type);
                self.lower_read_type(&underlying_hir_type, dest, format, field_map)?
            }
            HirType::UserDefined(name) => LirOperation::ReadStruct {
                dest,
                type_name: name.clone(),
            },
        })
    }

    fn lower_write_type(
        &mut self,
        ty: &HirType,
        src: VarId,
        format: &HirFormat,
        field_map: &HashMap<String, VarId>,
    ) -> Result<LirOperation, PipelineError> {
        Ok(match ty {
            HirType::U8 => LirOperation::WriteU8 { src },
            HirType::U16 => LirOperation::WriteU16 {
                src,
                endianness: format.endianness,
            },
            HirType::U32 => LirOperation::WriteU32 {
                src,
                endianness: format.endianness,
            },
            HirType::U64 => LirOperation::WriteU64 {
                src,
                endianness: format.endianness,
            },
            HirType::I8 => LirOperation::WriteI8 { src },
            HirType::I16 => LirOperation::WriteI16 {
                src,
                endianness: format.endianness,
            },
            HirType::I32 => LirOperation::WriteI32 {
                src,
                endianness: format.endianness,
            },
            HirType::I64 => LirOperation::WriteI64 {
                src,
                endianness: format.endianness,
            },
            // Bitfield types
            HirType::U1 => LirOperation::WriteBits { src, num_bits: 1 },
            HirType::U2 => LirOperation::WriteBits { src, num_bits: 2 },
            HirType::U3 => LirOperation::WriteBits { src, num_bits: 3 },
            HirType::U4 => LirOperation::WriteBits { src, num_bits: 4 },
            HirType::U5 => LirOperation::WriteBits { src, num_bits: 5 },
            HirType::U6 => LirOperation::WriteBits { src, num_bits: 6 },
            HirType::U7 => LirOperation::WriteBits { src, num_bits: 7 },
            HirType::I1 => LirOperation::WriteBits { src, num_bits: 1 },
            HirType::I2 => LirOperation::WriteBits { src, num_bits: 2 },
            HirType::I3 => LirOperation::WriteBits { src, num_bits: 3 },
            HirType::I4 => LirOperation::WriteBits { src, num_bits: 4 },
            HirType::I5 => LirOperation::WriteBits { src, num_bits: 5 },
            HirType::I6 => LirOperation::WriteBits { src, num_bits: 6 },
            HirType::I7 => LirOperation::WriteBits { src, num_bits: 7 },
            HirType::Array { element_type, size } => {
                let dummy_var = self.next_var();
                let element_op = self.lower_write_type(element_type, dummy_var, format, field_map)?;
                LirOperation::WriteArray {
                    src,
                    element_op: Box::new(element_op),
                    count: *size,
                }
            }
            HirType::DynamicArray { element_type, size_field } => {
                let size_var = *field_map.get(size_field).ok_or_else(|| {
                    PipelineError::UnknownType(format!("Size field '{}' not found", size_field))
                })?;
                let dummy_var = self.next_var();
                let element_op = self.lower_write_type(element_type, dummy_var, format, field_map)?;
                LirOperation::WriteDynamicArray {
                    src,
                    element_op: Box::new(element_op),
                    size_var,
                    size_field_name: size_field.clone(),
                }
            }
            HirType::UntilEofArray { element_type } => {
                let dummy_var = self.next_var();
                let element_op = self.lower_write_type(element_type, dummy_var, format, field_map)?;
                LirOperation::WriteUntilEofArray {
                    src,
                    element_op: Box::new(element_op),
                }
            }
            HirType::UntilConditionArray { element_type, .. } => {
                let dummy_var = self.next_var();
                let element_op = self.lower_write_type(element_type, dummy_var, format, field_map)?;
                LirOperation::WriteUntilConditionArray {
                    src,
                    element_op: Box::new(element_op),
                }
            }
            HirType::FixedString { size } => LirOperation::WriteFixedString {
                src,
                length: *size,
            },
            HirType::NullTerminatedString => LirOperation::WriteNullTerminatedString { src },
            HirType::LengthPrefixedString { length_field } => {
                let length_var = *field_map.get(length_field).ok_or_else(|| {
                    PipelineError::UnknownType(format!("Length field '{}' not found", length_field))
                })?;
                LirOperation::WriteLengthPrefixedString {
                    src,
                    length_var,
                }
            }
            HirType::Blob { .. } => {
                LirOperation::WriteBlob { src }
            }
            HirType::Enum(name) => {
                // Look up the enum definition
                let enum_def = format.enums.iter().find(|e| &e.name == name).ok_or_else(|| {
                    PipelineError::UnknownType(format!("Enum '{}' not found", name))
                })?;

                // Lower as the underlying primitive type
                let underlying_hir_type = primitive_to_hir_type(enum_def.underlying_type);
                self.lower_write_type(&underlying_hir_type, src, format, field_map)?
            }
            HirType::UserDefined(name) => LirOperation::WriteStruct {
                src,
                type_name: name.clone(),
            },
        })
    }

    fn next_var(&mut self) -> VarId {
        let id = self.next_var_id;
        self.next_var_id += 1;
        id
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

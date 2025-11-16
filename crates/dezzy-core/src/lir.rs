use crate::hir::Endianness;
use serde::{Deserialize, Serialize};

pub type VarId = usize;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LirFormat {
    pub name: String,
    pub types: Vec<LirType>,
    pub endianness: Endianness,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LirType {
    pub name: String,
    pub fields: Vec<LirField>,
    pub operations: Vec<LirOperation>,
    pub read_result: VarId,
    pub write_param: VarId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LirField {
    pub name: String,
    pub doc: Option<String>,
    pub var_id: VarId,
    pub type_info: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LirOperation {
    ReadU8 {
        dest: VarId,
    },
    ReadU16 {
        dest: VarId,
        endianness: Endianness,
    },
    ReadU32 {
        dest: VarId,
        endianness: Endianness,
    },
    ReadU64 {
        dest: VarId,
        endianness: Endianness,
    },
    ReadI8 {
        dest: VarId,
    },
    ReadI16 {
        dest: VarId,
        endianness: Endianness,
    },
    ReadI32 {
        dest: VarId,
        endianness: Endianness,
    },
    ReadI64 {
        dest: VarId,
        endianness: Endianness,
    },
    ReadArray {
        dest: VarId,
        element_op: Box<LirOperation>,
        count: usize,
    },
    ReadDynamicArray {
        dest: VarId,
        element_op: Box<LirOperation>,
        size_var: VarId,
    },
    ReadUntilEofArray {
        dest: VarId,
        element_op: Box<LirOperation>,
    },
    ReadStruct {
        dest: VarId,
        type_name: String,
    },
    WriteU8 {
        src: VarId,
    },
    WriteU16 {
        src: VarId,
        endianness: Endianness,
    },
    WriteU32 {
        src: VarId,
        endianness: Endianness,
    },
    WriteU64 {
        src: VarId,
        endianness: Endianness,
    },
    WriteI8 {
        src: VarId,
    },
    WriteI16 {
        src: VarId,
        endianness: Endianness,
    },
    WriteI32 {
        src: VarId,
        endianness: Endianness,
    },
    WriteI64 {
        src: VarId,
        endianness: Endianness,
    },
    WriteArray {
        src: VarId,
        element_op: Box<LirOperation>,
        count: usize,
    },
    WriteDynamicArray {
        src: VarId,
        element_op: Box<LirOperation>,
        size_var: VarId,
        size_field_name: String,
    },
    WriteUntilEofArray {
        src: VarId,
        element_op: Box<LirOperation>,
    },
    WriteStruct {
        src: VarId,
        type_name: String,
    },
    CreateStruct {
        dest: VarId,
        type_name: String,
        fields: Vec<VarId>,
    },
    AccessField {
        dest: VarId,
        struct_var: VarId,
        field_index: usize,
    },
}

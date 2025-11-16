use crate::expr::Expr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HirFormat {
    pub name: String,
    pub version: Option<String>,
    pub endianness: Endianness,
    pub enums: Vec<HirEnum>,
    pub types: Vec<HirTypeDef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HirEnum {
    pub name: String,
    pub doc: Option<String>,
    pub underlying_type: HirPrimitiveType,
    pub values: Vec<HirEnumValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HirEnumValue {
    pub name: String,
    pub value: i64,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HirPrimitiveType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HirTypeDef {
    Struct(HirStruct),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HirStruct {
    pub name: String,
    pub doc: Option<String>,
    pub fields: Vec<HirField>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HirField {
    pub name: String,
    pub doc: Option<String>,
    pub field_type: HirType,
    pub assertion: Option<HirAssertion>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HirAssertion {
    /// Value must equal the specified constant
    Equals(HirAssertValue),
    /// Value must not equal the specified constant
    NotEquals(HirAssertValue),
    /// Value must be greater than the threshold
    GreaterThan(i64),
    /// Value must be greater than or equal to the threshold
    GreaterOrEqual(i64),
    /// Value must be less than the threshold
    LessThan(i64),
    /// Value must be less than or equal to the threshold
    LessOrEqual(i64),
    /// Value must be in the specified set
    In(Vec<i64>),
    /// Value must not be in the specified set
    NotIn(Vec<i64>),
    /// Value must be in the range [min, max] inclusive
    Range { min: i64, max: i64 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HirAssertValue {
    /// Single integer value
    Int(i64),
    /// Array of integer values (for comparing against byte arrays)
    IntArray(Vec<i64>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HirType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    Array {
        element_type: Box<HirType>,
        size: usize,
    },
    DynamicArray {
        element_type: Box<HirType>,
        size_field: String,
    },
    UntilEofArray {
        element_type: Box<HirType>,
    },
    UntilConditionArray {
        element_type: Box<HirType>,
        condition: Expr,
    },
    Enum(String),
    UserDefined(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Endianness {
    Little,
    Big,
    Native,
}

impl HirType {
    pub fn size_in_bytes(&self) -> Option<usize> {
        match self {
            HirType::U8 | HirType::I8 => Some(1),
            HirType::U16 | HirType::I16 => Some(2),
            HirType::U32 | HirType::I32 => Some(4),
            HirType::U64 | HirType::I64 => Some(8),
            HirType::Array { element_type, size } => {
                element_type.size_in_bytes().map(|elem_size| elem_size * size)
            }
            HirType::DynamicArray { .. } => None,
            HirType::UntilEofArray { .. } => None,
            HirType::UntilConditionArray { .. } => None,
            HirType::Enum(_) => None, // Size determined by underlying type during lowering
            HirType::UserDefined(_) => None,
        }
    }

    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            HirType::U8
                | HirType::U16
                | HirType::U32
                | HirType::U64
                | HirType::I8
                | HirType::I16
                | HirType::I32
                | HirType::I64
        )
    }
}

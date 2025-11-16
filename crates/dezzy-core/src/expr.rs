use serde::{Deserialize, Serialize};

/// Expression AST for until conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    /// Field access: `chunks[-1].chunk_type`
    FieldAccess {
        base: Box<Expr>,
        field: String,
    },

    /// Array index: `chunks[-1]` or `items[0]`
    ArrayIndex {
        array: Box<Expr>,
        index: IndexExpr,
    },

    /// Variable reference: `chunks`, `packet`
    Variable(String),

    /// Binary comparison: `x equals y`, `x less-than y`
    Comparison {
        left: Box<Expr>,
        op: ComparisonOp,
        right: Box<Expr>,
    },

    /// Logical operation: `x AND y`, `x OR y`
    Logical {
        left: Box<Expr>,
        op: LogicalOp,
        right: Box<Expr>,
    },

    /// Literal values
    Literal(Literal),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IndexExpr {
    /// Positive index: `items[0]`
    Positive(usize),
    /// Negative index: `items[-1]` (from end)
    Negative(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComparisonOp {
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogicalOp {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    /// Integer literal: `42`, `0x2A`
    Integer(i64),

    /// Byte array literal: `[73, 69, 78, 68]`
    ByteArray(Vec<u8>),

    /// String literal converted to bytes: `'IEND'`
    String(String),
}

impl ComparisonOp {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "equals" => Some(ComparisonOp::Equals),
            "not-equals" => Some(ComparisonOp::NotEquals),
            "less-than" => Some(ComparisonOp::LessThan),
            "greater-than" => Some(ComparisonOp::GreaterThan),
            "less-than-or-equal" => Some(ComparisonOp::LessThanOrEqual),
            "greater-than-or-equal" => Some(ComparisonOp::GreaterThanOrEqual),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            ComparisonOp::Equals => "equals",
            ComparisonOp::NotEquals => "not-equals",
            ComparisonOp::LessThan => "less-than",
            ComparisonOp::GreaterThan => "greater-than",
            ComparisonOp::LessThanOrEqual => "less-than-or-equal",
            ComparisonOp::GreaterThanOrEqual => "greater-than-or-equal",
        }
    }
}

impl LogicalOp {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "AND" => Some(LogicalOp::And),
            "OR" => Some(LogicalOp::Or),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            LogicalOp::And => "AND",
            LogicalOp::Or => "OR",
        }
    }
}

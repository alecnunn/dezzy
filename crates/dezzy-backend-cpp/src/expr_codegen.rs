use anyhow::Result;
use dezzy_core::expr::{ComparisonOp, Expr, IndexExpr, Literal, LogicalOp};

/// Generate C++ code for an expression
#[must_use]
pub fn generate_expr(expr: &Expr, array_name: &str) -> Result<String> {
    Ok(match expr {
        Expr::Variable(name) => {
            // If the variable matches the last component of array_name, use array_name
            // For example, if array_name is "result.chunks" and name is "chunks", use "result.chunks"
            if let Some(last_component) = array_name.rsplit('.').next() {
                if name == last_component {
                    return Ok(array_name.to_string());
                }
            }
            // If array_name is not empty and doesn't already contain a dot (simple context),
            // prepend it to create a field reference (e.g., "result.version")
            if !array_name.is_empty() && !array_name.contains('.') {
                return Ok(format!("{}.{}", array_name, name));
            }
            name.clone()
        },

        Expr::FieldAccess { base, field } => {
            let base_code = generate_expr(base, array_name)?;
            format!("{}.{}", base_code, field)
        }

        Expr::ArrayIndex { array, index } => {
            let array_code = generate_expr(array, array_name)?;
            let index_code = match index {
                IndexExpr::Positive(i) => format!("[{}]", i),
                IndexExpr::Negative(i) => format!("[{}.size() - {}]", array_code, i),
            };
            format!("{}{}", array_code, index_code)
        }

        Expr::Comparison { left, op, right } => {
            let left_code = generate_expr(left, array_name)?;
            let right_code = generate_expr(right, array_name)?;
            let op_code = match op {
                ComparisonOp::Equals => "==",
                ComparisonOp::NotEquals => "!=",
                ComparisonOp::LessThan => "<",
                ComparisonOp::GreaterThan => ">",
                ComparisonOp::LessThanOrEqual => "<=",
                ComparisonOp::GreaterThanOrEqual => ">=",
            };

            // Special handling for comparing arrays/strings
            format!("({} {} {})", left_code, op_code, right_code)
        }

        Expr::Logical { left, op, right } => {
            let left_code = generate_expr(left, array_name)?;
            let right_code = generate_expr(right, array_name)?;
            let op_code = match op {
                LogicalOp::And => "&&",
                LogicalOp::Or => "||",
            };
            format!("({} {} {})", left_code, op_code, right_code)
        }

        Expr::Literal(lit) => generate_literal(lit)?,
    })
}

fn generate_literal(lit: &Literal) -> Result<String> {
    Ok(match lit {
        Literal::Integer(val) => val.to_string(),

        Literal::ByteArray(bytes) => {
            // Generate std::array<uint8_t, N>{...} initialization
            let elements: Vec<String> = bytes.iter().map(|b| b.to_string()).collect();
            format!("std::array<uint8_t, {}>{{{{{}}}}}",
                bytes.len(),
                elements.join(", "))
        }

        Literal::String(s) => {
            // Convert string to byte array
            let bytes: Vec<String> = s.bytes().map(|b| b.to_string()).collect();
            format!("std::array<uint8_t, {}>{{{{{}}}}}",
                bytes.len(),
                bytes.join(", "))
        }
    })
}

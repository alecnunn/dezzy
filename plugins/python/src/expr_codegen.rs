use dezzy_core::expr::{ComparisonOp, Expr, IndexExpr, Literal, LogicalOp};

/// Generate Python code for an expression
pub fn generate_expr(expr: &Expr, array_name: &str) -> Result<String, String> {
    Ok(match expr {
        Expr::Variable(name) => {
            // If the variable matches the array name (could have no dots), use array_name
            if array_name == name {
                return Ok(array_name.to_string());
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
                IndexExpr::Negative(i) => format!("[-{}]", i),
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
            format!("({} {} {})", left_code, op_code, right_code)
        }

        Expr::Logical { left, op, right } => {
            let left_code = generate_expr(left, array_name)?;
            let right_code = generate_expr(right, array_name)?;
            let op_code = match op {
                LogicalOp::And => "and",
                LogicalOp::Or => "or",
            };
            format!("({} {} {})", left_code, op_code, right_code)
        }

        Expr::Literal(lit) => generate_literal(lit)?,
    })
}

fn generate_literal(lit: &Literal) -> Result<String, String> {
    Ok(match lit {
        Literal::Integer(val) => val.to_string(),

        Literal::ByteArray(bytes) => {
            // Generate list of integers: [73, 69, 78, 68]
            let elements: Vec<String> = bytes.iter().map(|b| b.to_string()).collect();
            format!("[{}]", elements.join(", "))
        }

        Literal::String(s) => {
            // Convert string to list of byte values
            let bytes: Vec<String> = s.bytes().map(|b| b.to_string()).collect();
            format!("[{}]", bytes.join(", "))
        }
    })
}

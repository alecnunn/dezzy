use crate::error::ParseError;
use dezzy_core::expr::{ComparisonOp, Expr, IndexExpr, Literal, LogicalOp};

/// Parse an until condition expression
/// Examples:
///   - "chunks[-1].chunk_type equals 'IEND'"
///   - "chunks[-1].chunk_type equals [73, 69, 78, 68]"
///   - "packet.flags equals 0x00 AND packet.length less-than 1500"
pub fn parse_expr(input: &str) -> Result<Expr, ParseError> {
    let tokens = tokenize(input)?;
    parse_logical(&tokens, 0).map(|(expr, _)| expr)
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    Integer(i64),
    String(String),
    LeftBracket,
    RightBracket,
    Dot,
    Comma,
    Minus,
    CompOp(ComparisonOp),
    LogicalOp(LogicalOp),
}

fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            '[' => {
                tokens.push(Token::LeftBracket);
                chars.next();
            }
            ']' => {
                tokens.push(Token::RightBracket);
                chars.next();
            }
            '.' => {
                tokens.push(Token::Dot);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '\'' => {
                // String literal
                chars.next(); // consume opening quote
                let mut s = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '\'' {
                        chars.next(); // consume closing quote
                        break;
                    }
                    s.push(ch);
                    chars.next();
                }
                tokens.push(Token::String(s));
            }
            '0'..='9' => {
                let num_str = consume_while(&mut chars, |c| c.is_ascii_digit() || c == 'x' || c.is_ascii_hexdigit());

                let value = if num_str.starts_with("0x") || num_str.starts_with("0X") {
                    i64::from_str_radix(&num_str[2..], 16).map_err(|_| ParseError::InvalidValue {
                        field: "expression".to_string(),
                        message: format!("Invalid hex literal: {}", num_str),
                    })?
                } else {
                    num_str.parse().map_err(|_| ParseError::InvalidValue {
                        field: "expression".to_string(),
                        message: format!("Invalid integer: {}", num_str),
                    })?
                };
                tokens.push(Token::Integer(value));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let ident = consume_while(&mut chars, |c| c.is_alphanumeric() || c == '_' || c == '-');

                // Check for operators
                if let Some(op) = ComparisonOp::from_str(&ident) {
                    tokens.push(Token::CompOp(op));
                } else if let Some(op) = LogicalOp::from_str(&ident) {
                    tokens.push(Token::LogicalOp(op));
                } else {
                    tokens.push(Token::Ident(ident));
                }
            }
            _ => {
                return Err(ParseError::InvalidValue {
                    field: "expression".to_string(),
                    message: format!("Unexpected character: {}", ch),
                });
            }
        }
    }

    Ok(tokens)
}

fn consume_while<F>(chars: &mut std::iter::Peekable<std::str::Chars>, pred: F) -> String
where
    F: Fn(char) -> bool,
{
    let mut result = String::new();
    while let Some(&ch) = chars.peek() {
        if pred(ch) {
            result.push(ch);
            chars.next();
        } else {
            break;
        }
    }
    result
}

// Recursive descent parser for expressions
// Priority: Logical (lowest) > Comparison > Primary (highest)

fn parse_logical(tokens: &[Token], pos: usize) -> Result<(Expr, usize), ParseError> {
    let (mut left, mut pos) = parse_comparison(tokens, pos)?;

    while pos < tokens.len() {
        if let Token::LogicalOp(op) = &tokens[pos] {
            let op = op.clone();
            pos += 1;
            let (right, new_pos) = parse_comparison(tokens, pos)?;
            pos = new_pos;
            left = Expr::Logical {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        } else {
            break;
        }
    }

    Ok((left, pos))
}

fn parse_comparison(tokens: &[Token], pos: usize) -> Result<(Expr, usize), ParseError> {
    let (left, mut pos) = parse_primary(tokens, pos)?;

    if pos < tokens.len() {
        if let Token::CompOp(op) = &tokens[pos] {
            let op = op.clone();
            pos += 1;
            let (right, new_pos) = parse_primary(tokens, pos)?;
            pos = new_pos;
            return Ok((
                Expr::Comparison {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                pos,
            ));
        }
    }

    Ok((left, pos))
}

fn parse_primary(tokens: &[Token], mut pos: usize) -> Result<(Expr, usize), ParseError> {
    if pos >= tokens.len() {
        return Err(ParseError::InvalidValue {
            field: "expression".to_string(),
            message: "Unexpected end of expression".to_string(),
        });
    }

    let mut expr = match &tokens[pos] {
        Token::Ident(name) => {
            pos += 1;
            Expr::Variable(name.clone())
        }
        Token::Integer(val) => {
            pos += 1;
            Expr::Literal(Literal::Integer(*val))
        }
        Token::String(s) => {
            pos += 1;
            Expr::Literal(Literal::String(s.clone()))
        }
        Token::LeftBracket => {
            // Byte array literal: [73, 69, 78, 68]
            pos += 1;
            let mut bytes = Vec::new();
            while pos < tokens.len() && tokens[pos] != Token::RightBracket {
                if let Token::Integer(val) = &tokens[pos] {
                    bytes.push(*val as u8);
                    pos += 1;
                    if pos < tokens.len() && tokens[pos] == Token::Comma {
                        pos += 1; // skip comma
                    }
                } else {
                    return Err(ParseError::InvalidValue {
                        field: "expression".to_string(),
                        message: "Expected integer in byte array".to_string(),
                    });
                }
            }
            if pos >= tokens.len() || tokens[pos] != Token::RightBracket {
                return Err(ParseError::InvalidValue {
                    field: "expression".to_string(),
                    message: "Expected closing bracket".to_string(),
                });
            }
            pos += 1; // skip ]
            Expr::Literal(Literal::ByteArray(bytes))
        }
        _ => {
            return Err(ParseError::InvalidValue {
                field: "expression".to_string(),
                message: format!("Unexpected token: {:?}", tokens[pos]),
            });
        }
    };

    // Handle postfix operations: array indexing and field access
    loop {
        if pos >= tokens.len() {
            break;
        }

        match &tokens[pos] {
            Token::LeftBracket => {
                // Array indexing
                pos += 1;
                let is_negative = if pos < tokens.len() && tokens[pos] == Token::Minus {
                    pos += 1;
                    true
                } else {
                    false
                };

                if pos >= tokens.len() {
                    return Err(ParseError::InvalidValue {
                        field: "expression".to_string(),
                        message: "Expected index".to_string(),
                    });
                }

                let index = if let Token::Integer(val) = &tokens[pos] {
                    pos += 1;
                    *val as usize
                } else {
                    return Err(ParseError::InvalidValue {
                        field: "expression".to_string(),
                        message: "Expected integer index".to_string(),
                    });
                };

                if pos >= tokens.len() || tokens[pos] != Token::RightBracket {
                    return Err(ParseError::InvalidValue {
                        field: "expression".to_string(),
                        message: "Expected closing bracket".to_string(),
                    });
                }
                pos += 1;

                expr = Expr::ArrayIndex {
                    array: Box::new(expr),
                    index: if is_negative {
                        IndexExpr::Negative(index)
                    } else {
                        IndexExpr::Positive(index)
                    },
                };
            }
            Token::Dot => {
                // Field access
                pos += 1;
                if pos >= tokens.len() {
                    return Err(ParseError::InvalidValue {
                        field: "expression".to_string(),
                        message: "Expected field name after '.'".to_string(),
                    });
                }

                let field = if let Token::Ident(name) = &tokens[pos] {
                    pos += 1;
                    name.clone()
                } else {
                    return Err(ParseError::InvalidValue {
                        field: "expression".to_string(),
                        message: "Expected field name".to_string(),
                    });
                };

                expr = Expr::FieldAccess {
                    base: Box::new(expr),
                    field,
                };
            }
            _ => break,
        }
    }

    Ok((expr, pos))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_comparison() {
        let expr = parse_expr("x equals 5").unwrap();
        match expr {
            Expr::Comparison { left, op, right } => {
                assert!(matches!(*left, Expr::Variable(_)));
                assert_eq!(op, ComparisonOp::Equals);
                assert!(matches!(*right, Expr::Literal(Literal::Integer(5))));
            }
            _ => panic!("Expected comparison"),
        }
    }

    #[test]
    fn test_parse_field_access() {
        let expr = parse_expr("chunk.type equals 'IEND'").unwrap();
        match expr {
            Expr::Comparison { left, op, right } => {
                assert!(matches!(*left, Expr::FieldAccess { .. }));
                assert_eq!(op, ComparisonOp::Equals);
                assert!(matches!(*right, Expr::Literal(Literal::String(_))));
            }
            _ => panic!("Expected comparison"),
        }
    }

    #[test]
    fn test_parse_array_index() {
        let expr = parse_expr("chunks[-1].chunk_type equals [73, 69, 78, 68]").unwrap();
        match expr {
            Expr::Comparison { left, op, right } => {
                assert_eq!(op, ComparisonOp::Equals);
                assert!(matches!(*right, Expr::Literal(Literal::ByteArray(_))));
            }
            _ => panic!("Expected comparison"),
        }
    }
}

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]

mod error;
mod expr_parser;
mod schema;
mod parser;

pub use error::ParseError;
pub use expr_parser::parse_expr;
pub use parser::parse_format;

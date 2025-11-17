#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]

mod codegen;
mod expr_codegen;
mod templates;

pub use codegen::CppBackend;

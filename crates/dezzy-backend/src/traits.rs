use anyhow::Result;
use dezzy_core::lir::LirFormat;

#[derive(Debug, Clone)]
pub struct GeneratedCode {
    pub files: Vec<GeneratedFile>,
}

#[derive(Debug, Clone)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
}

pub trait Backend {
    fn name(&self) -> &str;
    fn generate(&self, lir: &LirFormat) -> Result<GeneratedCode>;
}

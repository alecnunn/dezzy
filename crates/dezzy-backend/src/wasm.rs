use crate::traits::{Backend, GeneratedCode};
use anyhow::Result;
use dezzy_core::lir::LirFormat;

pub struct WasmBackend {
    name: String,
}

impl WasmBackend {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn from_file(_path: &str) -> Result<Self> {
        todo!("WASM plugin loading will be implemented in phase 2")
    }
}

impl Backend for WasmBackend {
    fn name(&self) -> &str {
        &self.name
    }

    fn generate(&self, _lir: &LirFormat) -> Result<GeneratedCode> {
        todo!("WASM plugin execution will be implemented in phase 2")
    }
}

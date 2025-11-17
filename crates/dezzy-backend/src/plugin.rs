use crate::traits::{Backend, GeneratedCode};
use anyhow::Result;
use dezzy_core::lir::LirFormat;
use std::collections::HashMap;
use std::sync::Arc;

pub struct PluginRegistry {
    backends: HashMap<String, Arc<dyn Backend + Send + Sync>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            backends: HashMap::new(),
        }
    }

    pub fn register(&mut self, backend: Arc<dyn Backend + Send + Sync>) {
        let name = backend.name().to_string();
        self.backends.insert(name, backend);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Backend + Send + Sync>> {
        self.backends.get(name).cloned()
    }

    pub fn list_backends(&self) -> Vec<String> {
        self.backends.keys().cloned().collect()
    }

    #[must_use]
    pub fn generate(&self, backend_name: &str, lir: &LirFormat) -> Result<GeneratedCode> {
        let backend = self
            .get(backend_name)
            .ok_or_else(|| anyhow::anyhow!("Backend '{}' not found", backend_name))?;

        backend.generate(lir)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

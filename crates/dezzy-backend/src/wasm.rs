use crate::traits::{Backend, GeneratedCode, GeneratedFile};
use anyhow::{Context, Result};
use dezzy_core::lir::LirFormat;
use serde_json;
use std::path::{Path, PathBuf};
use wasmer::{Instance, Module, Store, imports};

/// WASM plugin backend that implements the dezzy:backend interface
pub struct WasmBackend {
    backend_name: String,
    version: String,
    file_extension: String,
    wasm_path: PathBuf,
    wasm_bytes: Vec<u8>,
}

impl WasmBackend {
    pub fn new(name: String) -> Self {
        Self {
            backend_name: name,
            version: "0.1.0".to_string(),
            file_extension: "unknown".to_string(),
            wasm_path: PathBuf::new(),
            wasm_bytes: Vec::new(),
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let wasm_bytes = std::fs::read(path)
            .context(format!("Failed to read WASM file: {}", path.display()))?;

        // Create a temporary instance to read metadata
        let mut store = Store::default();
        let module = Module::new(&store, &wasm_bytes)
            .context("Failed to compile WASM module")?;

        // Instantiate the module
        let imports = imports! {};
        let instance = Instance::new(&mut store, &module, &imports)
            .context("Failed to instantiate WASM module")?;

        // Call metadata functions to get backend info
        let backend_name = Self::call_string_fn(&instance, &mut store, "get_name")?;
        let version = Self::call_string_fn(&instance, &mut store, "get_version")?;
        let file_extension = Self::call_string_fn(&instance, &mut store, "get_file_extension")?;

        Ok(Self {
            backend_name,
            version,
            file_extension,
            wasm_path: path.to_path_buf(),
            wasm_bytes,
        })
    }

    fn call_string_fn(instance: &Instance, store: &mut Store, fn_name: &str) -> Result<String> {
        // For this initial implementation, we expect a simple ABI where:
        // - Strings are returned as (ptr: i32, len: i32)
        // - The function allocates and returns a pointer to memory

        let func = instance.exports.get_function(fn_name)
            .context(format!("Function '{}' not found in WASM module", fn_name))?;

        let result = func.call(store, &[])
            .context(format!("Failed to call {}", fn_name))?;

        // Expect (ptr, len) return
        if result.len() < 2 {
            anyhow::bail!("{} returned invalid result (expected ptr and len)", fn_name);
        }

        let ptr = result[0].i32().ok_or_else(|| anyhow::anyhow!("Invalid pointer"))?;
        let len = result[1].i32().ok_or_else(|| anyhow::anyhow!("Invalid length"))?;

        // Read from WASM linear memory
        let memory = instance.exports.get_memory("memory")
            .context("WASM module doesn't export memory")?;

        let view = memory.view(store);
        let mut buffer = vec![0u8; len as usize];
        view.read(ptr as u64, &mut buffer)
            .context("Failed to read from WASM memory")?;

        String::from_utf8(buffer)
            .context("Invalid UTF-8 in WASM string")
    }

    fn call_generate(&self, lir_json: &str) -> Result<String> {
        // Create a fresh instance for each generation
        let mut store = Store::default();
        let module = Module::new(&store, &self.wasm_bytes)
            .context("Failed to compile WASM module")?;

        let imports = imports! {};
        let instance = Instance::new(&mut store, &module, &imports)
            .context("Failed to instantiate WASM module")?;

        // Allocate space in WASM memory for the LIR JSON
        let alloc_fn = instance.exports.get_function("alloc")
            .context("WASM module doesn't export 'alloc' function")?;

        let json_len = lir_json.len() as i32;
        let result = alloc_fn.call(&mut store, &[json_len.into()])
            .context("Failed to allocate memory in WASM")?;

        let ptr = result[0].i32().ok_or_else(|| anyhow::anyhow!("Invalid allocation pointer"))?;

        // Write LIR JSON to WASM memory
        let memory = instance.exports.get_memory("memory")
            .context("WASM module doesn't export memory")?;

        {
            let view = memory.view(&store);
            view.write(ptr as u64, lir_json.as_bytes())
                .context("Failed to write to WASM memory")?;
        } // Drop view to release immutable borrow

        // Call generate function
        let generate_fn = instance.exports.get_function("generate")
            .context("WASM module doesn't export 'generate' function")?;

        let result = generate_fn.call(&mut store, &[ptr.into(), json_len.into()])
            .context("Failed to call generate")?;

        // Read result (ptr, len)
        let result_ptr = result[0].i32().ok_or_else(|| anyhow::anyhow!("Invalid result pointer"))?;
        let result_len = result[1].i32().ok_or_else(|| anyhow::anyhow!("Invalid result length"))?;

        let mut output = vec![0u8; result_len as usize];
        {
            let view = memory.view(&store);
            view.read(result_ptr as u64, &mut output)
                .context("Failed to read generated code from WASM memory")?;
        }

        String::from_utf8(output)
            .context("Invalid UTF-8 in generated code")
    }
}

impl Backend for WasmBackend {
    fn name(&self) -> &str {
        &self.backend_name
    }

    fn generate(&self, lir: &LirFormat) -> Result<GeneratedCode> {
        // Serialize LIR to JSON
        let lir_json = serde_json::to_string(lir)
            .context("Failed to serialize LIR to JSON")?;

        // Call the WASM plugin's generate function
        let generated_code = self.call_generate(&lir_json)
            .context("WASM plugin generate() failed")?;

        Ok(GeneratedCode {
            files: vec![GeneratedFile {
                path: format!("{}.{}", lir.name.to_lowercase(), self.file_extension),
                content: generated_code,
            }],
        })
    }
}

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dezzy_backend::{PluginRegistry, WasmBackend};
use dezzy_backend_cpp::CppBackend;
use dezzy_core::pipeline::Pipeline;
use dezzy_parser::parse_format;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "dezzy")]
#[command(about = "A DSL for binary format parsing and SDK generation", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Compile {
        #[arg(help = "Input format definition file")]
        input: String,

        #[arg(short, long, help = "Backend to use for code generation")]
        backend: String,

        #[arg(short, long, help = "Output directory")]
        output: String,
    },
    Validate {
        #[arg(help = "Input format definition file")]
        input: String,
    },
    #[command(about = "List all available code generation backends")]
    ListBackends,
}

#[derive(Debug, Deserialize)]
struct PluginMetadata {
    name: String,
    version: String,
    path: String,
    description: String,
    author: String,
}

#[derive(Debug, Deserialize)]
struct PluginManifest {
    plugin: Vec<PluginMetadata>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile {
            input,
            backend,
            output,
        } => compile_command(&input, &backend, &output),
        Commands::Validate { input } => validate_command(&input),
        Commands::ListBackends => list_backends_command(),
    }
}

fn discover_backends() -> Result<PluginRegistry> {
    let mut registry = PluginRegistry::new();

    // Register built-in C++ backend
    registry.register(Arc::new(CppBackend::new()));

    // Try to load plugin manifest
    let manifest_path = PathBuf::from("plugins/manifest.toml");
    if !manifest_path.exists() {
        // No plugins, that's fine - just use built-in backends
        return Ok(registry);
    }

    let manifest_content = fs::read_to_string(&manifest_path)
        .context("Failed to read plugin manifest")?;

    let manifest: PluginManifest = toml::from_str(&manifest_content)
        .context("Failed to parse plugin manifest")?;

    // Load each plugin
    for plugin_meta in manifest.plugin {
        let plugin_path = PathBuf::from("plugins").join(&plugin_meta.path);

        match WasmBackend::from_file(&plugin_path) {
            Ok(backend) => {
                println!("Loaded plugin: {} v{}", plugin_meta.name, plugin_meta.version);
                registry.register(Arc::new(backend));
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to load plugin '{}': {}",
                    plugin_meta.name, e
                );
            }
        }
    }

    Ok(registry)
}

fn list_backends_command() -> Result<()> {
    println!("Available backends:\n");

    // Built-in backends
    println!("Built-in:");
    println!("  cpp - C++ header-only code generator");

    // Plugin backends
    let manifest_path = PathBuf::from("plugins/manifest.toml");
    if manifest_path.exists() {
        let manifest_content = fs::read_to_string(&manifest_path)?;
        let manifest: PluginManifest = toml::from_str(&manifest_content)?;

        if !manifest.plugin.is_empty() {
            println!("\nPlugins:");
            for plugin in manifest.plugin {
                println!("  {} (v{}) - {}", plugin.name, plugin.version, plugin.description);
            }
        }
    }

    Ok(())
}

fn compile_command(input_path: &str, backend_name: &str, output_spec: &str) -> Result<()> {
    let yaml_content = fs::read_to_string(input_path)
        .with_context(|| format!("Failed to read input file: {}", input_path))?;

    let hir_format = match parse_format(&yaml_content) {
        Ok(format) => format,
        Err(e) => {
            eprintln!("Error parsing format definition:");
            eprintln!("{}", e);
            return Err(e.into());
        }
    };

    println!("Parsed format: {}", hir_format.name);

    let mut pipeline = Pipeline::new();
    let lir_format = pipeline
        .lower(hir_format)
        .context("Failed to lower HIR to LIR")?;

    println!("Lowered to LIR with {} types", lir_format.types.len());

    // Discover and load all available backends
    let registry = discover_backends()
        .context("Failed to discover backends")?;

    let generated = registry
        .generate(backend_name, &lir_format)
        .with_context(|| format!("Backend '{}' failed to generate code", backend_name))?;

    let output_path = Path::new(output_spec);

    // Determine if output_spec is a file or directory
    // If it has an extension, treat it as a file path
    let (output_dir, override_filename) = if output_path.extension().is_some() {
        // It's a file path - extract directory and filename
        let dir = output_path.parent().unwrap_or_else(|| Path::new("."));
        let filename = output_path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());
        (dir, filename)
    } else {
        // It's a directory
        (output_path, None)
    };

    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;

    for file in generated.files {
        let filename = override_filename.as_ref().unwrap_or(&file.path);
        let file_path = output_dir.join(filename);
        fs::write(&file_path, file.content)
            .with_context(|| format!("Failed to write output file: {}", file_path.display()))?;
        println!("Generated: {}", file_path.display());
    }

    println!("Compilation successful!");

    Ok(())
}

fn validate_command(input_path: &str) -> Result<()> {
    let yaml_content = fs::read_to_string(input_path)
        .with_context(|| format!("Failed to read input file: {}", input_path))?;

    match parse_format(&yaml_content) {
        Ok(format) => {
            println!("Validation successful!");
            println!("Format: {}", format.name);
            println!("Version: {:?}", format.version);
            println!("Endianness: {:?}", format.endianness);
            println!("Types: {}", format.types.len());

            for type_def in &format.types {
                match type_def {
                    dezzy_core::hir::HirTypeDef::Struct(s) => {
                        println!("  - {} (struct with {} fields)", s.name, s.fields.len());
                    }
                }
            }

            Ok(())
        }
        Err(e) => {
            eprintln!("Validation failed!");
            eprintln!("{}", e);
            Err(e.into())
        }
    }
}

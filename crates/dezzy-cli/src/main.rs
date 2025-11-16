use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dezzy_backend::PluginRegistry;
use dezzy_backend_cpp::CppBackend;
use dezzy_core::pipeline::Pipeline;
use dezzy_parser::parse_format;
use std::fs;
use std::path::Path;
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
    }
}

fn compile_command(input_path: &str, backend_name: &str, output_dir: &str) -> Result<()> {
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

    let mut registry = PluginRegistry::new();
    registry.register(Arc::new(CppBackend::new()));

    let generated = registry
        .generate(backend_name, &lir_format)
        .with_context(|| format!("Backend '{}' failed to generate code", backend_name))?;

    let output_path = Path::new(output_dir);
    fs::create_dir_all(output_path)
        .with_context(|| format!("Failed to create output directory: {}", output_dir))?;

    for file in generated.files {
        let file_path = output_path.join(&file.path);
        fs::write(&file_path, file.content)
            .with_context(|| format!("Failed to write output file: {}", file.path))?;
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

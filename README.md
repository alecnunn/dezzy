# Dezzy

A modern DSL for describing binary file formats and generating type-safe SDKs for reading and writing those formats.

## Overview

Dezzy is an alternative to Kaitai Struct, written in Rust, that provides:
- YAML-based format definitions
- Multi-stage IR compilation pipeline
- Extensible backend system via WASM plugins
- Header-only C++17 code generation (with more backends planned)

## Features

### Current (v0.1)
- Basic type definitions (u8, u16, u32, u64, i8, i16, i32, i64)
- Fixed-size arrays
- Struct types with full nesting support
- Automatic dependency ordering (topological sort)
- Circular dependency detection
- Explicit endianness control (little, big, native)
- Read and write operations
- C++20 backend with header-only output

### Planned
- Conditional parsing
- Computed fields and expressions
- Enums and type unions
- Validation/assertions
- Additional backends (Python, Rust, TypeScript)
- WASM plugin support for custom backends

## Quick Start

### Installation

Build from source:
```bash
cargo build --release
```

The binary will be at `target/release/dezzy.exe`

### Usage

#### Validate a format definition
```bash
dezzy validate examples/simple.yaml
```

#### Compile a format to C++
```bash
dezzy compile examples/simple.yaml --backend cpp --output generated/
```

## Format Definition Example

### Simple Format
```yaml
name: SimpleFormat
version: "1.0"
endianness: little

types:
  - name: Header
    type: struct
    doc: "File header containing magic number and version"
    fields:
      - name: magic
        type: u32
        doc: "Magic number identifying the file format"
      - name: version
        type: u16
        doc: "Format version number"
      - name: flags
        type: u16
        doc: "Format flags"

  - name: DataBlock
    type: struct
    doc: "Data block with fixed-size payload"
    fields:
      - name: length
        type: u32
        doc: "Length of the data block"
      - name: data
        type: u8[16]
        doc: "Fixed-size data payload"
```

### Nested Structs
```yaml
name: NestedFormat
version: "1.0"
endianness: little

types:
  - name: Point
    type: struct
    fields:
      - name: x
        type: u32
      - name: y
        type: u32

  - name: Rectangle
    type: struct
    fields:
      - name: top_left
        type: Point
      - name: bottom_right
        type: Point
      - name: color
        type: u32
```

## Generated Code Example

The nested format generates header-only C++20 code with proper dependency ordering:

```cpp
namespace nestedformat {

struct Point {
    uint32_t x;
    uint32_t y;

    static Point read(Reader& reader);
    void write(Writer& writer) const;
};

struct Rectangle {
    Point top_left;      // Nested struct
    Point bottom_right;  // Nested struct
    uint32_t color;

    static Rectangle read(Reader& reader);
    void write(Writer& writer) const;
};

} // namespace nestedformat
```

Types are automatically ordered so dependencies come first, even if defined in reverse order in the YAML.

## Architecture

Dezzy uses a multi-stage compilation pipeline:

1. **YAML DSL** → Parser
2. **High-level IR (HIR)** → Semantic analysis
3. **Low-level IR (LIR)** → Backend-agnostic operations
4. **Code Generation** → Language-specific output

## Type System

### Primitive Types
- `u8`, `u16`, `u32`, `u64` - Unsigned integers
- `i8`, `i16`, `i32`, `i64` - Signed integers

### Array Types
- `type[size]` - Fixed-size arrays (e.g., `u8[16]`)

### Struct Types
- User-defined composite types
- Can reference other structs

### Endianness
- `little` - Little-endian (default)
- `big` - Big-endian
- `native` - Platform native

## Development

### Building
```bash
cargo build
```

### Running Tests
```bash
cargo test
```

### Running the CLI
```bash
cargo run -- validate examples/simple.yaml
cargo run -- compile examples/simple.yaml --backend cpp --output generated/
```

## License

MIT

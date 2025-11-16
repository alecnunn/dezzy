# Dezzy Architecture

## Overview
Dezzy is a declarative DSL for describing binary file formats and generating type-safe SDKs for reading and writing those formats. The system uses a multi-stage IR pipeline with WASM-based backend plugins for extensibility.

## System Architecture

```
┌──────────────┐
│  YAML DSL    │ (User-written format definitions)
└──────┬───────┘
       │
       v
┌──────────────────┐
│  dezzy-parser    │ (Parse YAML, validate schema)
└──────┬───────────┘
       │
       v
┌──────────────────┐
│  High-level IR   │ (Semantic model with type info)
└──────┬───────────┘
       │
       v
┌──────────────────┐
│  dezzy-core      │ (Transformations, lowering)
└──────┬───────────┘
       │
       v
┌──────────────────┐
│  Low-level IR    │ (Backend-agnostic operations)
└──────┬───────────┘
       │
       v
┌──────────────────┐
│  dezzy-backend   │ (Plugin system, WASM runtime)
└──────┬───────────┘
       │
       ├─────────────────────────────────┐
       v                                 v
┌──────────────────┐            ┌─────────────────┐
│ dezzy-backend-   │            │  WASM Plugin    │
│      cpp         │            │  (Future)       │
│  (Native)        │            │                 │
└──────┬───────────┘            └─────────────────┘
       │
       v
┌──────────────────┐
│  Generated Code  │ (C++ header-only library)
└──────────────────┘
```

## Component Details

### dezzy-parser
**Responsibilities:**
- Parse YAML format definitions
- Validate schema structure
- Convert to high-level IR
- Provide detailed error diagnostics (using ariadne)

**Input:** YAML file
**Output:** High-level IR (HirFormat)

### dezzy-core
**Responsibilities:**
- Define IR types (both high-level and low-level)
- Implement multi-stage transformation pipeline
- Type checking and semantic analysis
- IR lowering (HIR → LIR)
- Provide common utilities

**Key Types:**
- `HirFormat`: High-level intermediate representation
- `LirFormat`: Low-level intermediate representation
- Pipeline traits and transformation passes

### dezzy-backend
**Responsibilities:**
- Define backend plugin API/ABI
- WASM runtime integration (wasmtime)
- Plugin discovery and loading
- Backend trait abstraction

**Plugin Interface:**
```rust
trait Backend {
    fn name(&self) -> &str;
    fn generate(&self, lir: &LirFormat) -> Result<GeneratedCode>;
}
```

### dezzy-backend-cpp
**Responsibilities:**
- Generate C++17 header-only code
- Implement read operations (deserialization)
- Implement write operations (serialization)
- Use STL types (std::vector, std::array, std::unique_ptr, etc.)
- Handle endianness conversions

**Output:**
- Single .hpp file per format
- Type-safe parsing/serialization API
- Error handling using std::expected or Result types

### dezzy-cli
**Responsibilities:**
- Command-line interface (using clap)
- File I/O and orchestration
- Error reporting

**Commands:**
- `dezzy compile <input.yaml> --backend cpp --output <dir>`
- `dezzy validate <input.yaml>`

## Intermediate Representations

### High-Level IR (HIR)
Closely maps to the DSL structure with full type information and semantic details.

```rust
struct HirFormat {
    name: String,
    version: Option<String>,
    endianness: Endianness,
    types: Vec<HirTypeDef>,
}

enum HirTypeDef {
    Struct(HirStruct),
    // Future: Enum, Union, etc.
}

struct HirStruct {
    name: String,
    fields: Vec<HirField>,
}

struct HirField {
    name: String,
    field_type: HirType,
}

enum HirType {
    U8, U16, U32, U64,
    I8, I16, I32, I64,
    Array { element_type: Box<HirType>, size: usize },
    UserDefined(String),
}

enum Endianness {
    Little,
    Big,
    Native,
}
```

### Low-Level IR (LIR)
Backend-agnostic representation focused on operations.

```rust
struct LirFormat {
    name: String,
    operations: Vec<LirOperation>,
}

enum LirOperation {
    ReadBytes { dest: VarId, count: usize },
    WriteBytes { src: VarId, count: usize },
    ConvertEndian { var: VarId, from: Endianness, to: Endianness },
    CreateStruct { type_name: String, fields: Vec<VarId> },
    // etc.
}
```

## DSL Example

```yaml
name: SimpleFormat
version: "1.0"
endianness: little

types:
  - name: Header
    type: struct
    fields:
      - name: magic
        type: u32
      - name: version
        type: u16
      - name: flags
        type: u16

  - name: DataBlock
    type: struct
    fields:
      - name: length
        type: u32
      - name: data
        type: u8[16]  # Fixed-size array
```

## Generated Code Example (C++)

```cpp
#pragma once
#include <cstdint>
#include <array>
#include <vector>
#include <span>
#include <expected>

namespace simple_format {

struct Header {
    uint32_t magic;
    uint16_t version;
    uint16_t flags;

    static std::expected<Header, std::string> read(std::span<const uint8_t> data);
    std::vector<uint8_t> write() const;
};

struct DataBlock {
    uint32_t length;
    std::array<uint8_t, 16> data;

    static std::expected<DataBlock, std::string> read(std::span<const uint8_t> data);
    std::vector<uint8_t> write() const;
};

} // namespace simple_format
```

## Error Handling

### Compile-Time (DSL Parsing)
- Use `ariadne` for beautiful error diagnostics
- Show source location, error message, and suggestions
- Example:
```
Error: Invalid type reference
  ┌─ format.yaml:15:14
  │
15│         type: InvalidType
  │               ^^^^^^^^^^^ unknown type 'InvalidType'
  │
  = note: available types: u8, u16, u32, u64, i8, i16, i32, i64, Header
```

### Runtime (Generated Code)
- C++ backend uses `std::expected<T, E>` for fallible operations
- Clear error messages for parse failures
- Position tracking for debugging

## Plugin System

### WASM Plugin ABI
Plugins receive serialized LIR and return generated code.

**Functions exported by plugin:**
```rust
#[no_mangle]
pub extern "C" fn backend_name() -> *const c_char;

#[no_mangle]
pub extern "C" fn generate(lir_ptr: *const u8, lir_len: usize) -> *const GeneratedOutput;
```

### Plugin Discovery
- Scan `~/.dezzy/plugins/` directory
- Look for `.wasm` files
- Load using wasmtime runtime
- Cache plugin instances

## Build and Development

### Workspace Structure
```
dezzy/
├── Cargo.toml              (workspace root)
├── crates/
│   ├── dezzy-core/         (IR and pipeline)
│   ├── dezzy-parser/       (YAML parsing)
│   ├── dezzy-backend/      (plugin system)
│   ├── dezzy-backend-cpp/  (C++ codegen)
│   └── dezzy-cli/          (CLI tool)
├── docs/                   (documentation)
├── tests/
│   └── integration/        (end-to-end tests)
└── examples/               (sample format definitions)
```

### Testing Strategy
1. **Unit tests**: Each crate tests its own functionality
2. **Integration tests**: End-to-end format compilation
3. **Golden tests**: Compare generated code against expected output
4. **Format tests**: Test with real formats (e.g., simple binary formats)

## Future Extensions

### Phase 2 Features
- Conditional parsing (`if` statements)
- Computed fields and expressions
- Enums and type unions
- Validation and assertions

### Plugin Ecosystem
- Python backend
- Rust backend
- TypeScript backend
- Custom user-written backends

## Security Considerations
- WASM sandboxing prevents malicious plugins from accessing filesystem
- Input validation in parser prevents injection attacks
- Safe Rust code prevents memory safety issues

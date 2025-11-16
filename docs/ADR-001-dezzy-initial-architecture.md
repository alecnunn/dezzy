# ADR-001: Dezzy Initial Architecture

## Status
PROPOSED - Awaiting answers

## Context
We are designing and implementing dezzy, a DSL for decoding file or protocol formats with SDK generation capabilities. It aims to be an alternative to Kaitai Struct with advanced capabilities, written in Rust with a WASM-based plugin system for language backends.

## Decision Points

### 1. DSL Syntax and Format
**Question:** What syntax style should the dezzy DSL use?
- YAML-based (like Kaitai Struct)?
- Custom declarative syntax (like Protocol Buffers)?
- S-expression based?
- Other preference?

**Answer:** YAML-based

---

### 2. Core Feature Set (Phase 1)
**Question:** Which core features should be prioritized for the initial implementation?
- Basic type definitions (integers, strings, arrays)?
- Conditional parsing (if statements)?
- Computed fields and expressions?
- Enums and type unions?
- Validation/assertions?
- Which subset should we focus on first?

**Answer:** Basic type definitions

---

### 3. Intermediate Representation (IR)
**Question:** How should we structure the IR between DSL parsing and code generation?
- AST directly from parser?
- Separate typed IR with semantic analysis?
- Multi-stage IR (high-level → low-level)?

**Answer:** Multi-stage IR

---

### 4. Expression Language
**Question:** Should dezzy have an expression language for computed fields, sizes, conditions, etc.?
- Simple expressions only (arithmetic, comparisons)?
- Full expression language with functions?
- What operations are essential?

**Answer:** Simple only for the time being

---

### 5. Backend Plugin Architecture
**Question:** How should the WASM plugin system work?
- Plugin API design: What interface should plugins implement?
- How should plugins receive the IR?
- Should we support both WASM and native (compiled Rust) backends?
- Plugin discovery mechanism?

**Answer:** The plugins should specifically focus on language/codegen backends. Only focus on WASM plugins. Plugin discovery can be looking for specific files (wasm binaries) in a given directory, pretty standard stuff.

---

### 6. C++ Backend Specifics
**Question:** For the C++ code generator, what should the generated code look like?
- Header-only vs separate .h/.cpp?
- Modern C++ version target (C++17, C++20, C++23)?
- Memory management approach (raw pointers, unique_ptr, shared_ptr)?
- Serialization/deserialization or read-only?
- STL usage vs custom containers?

**Answer:** Try to generate header-only code, C++17, whatever makes the most sense for a library that should be safe, read & write, stl usage is fine

---

### 7. Error Handling Strategy
**Question:** How should dezzy handle errors?
- DSL compilation errors: How detailed should diagnostics be?
- Runtime parsing errors in generated code: Exceptions? Result types? Error codes?
- Should we prioritize helpful error messages from the start?

**Answer:** We should try to use ariadne for error details

---

### 8. CLI Design
**Question:** What should the dezzy CLI tool do?
- `dezzy compile <input.dz> --backend cpp --output ./gen/`?
- `dezzy validate <input.dz>`?
- `dezzy format <input.dz>`?
- Plugin management commands?
- What's essential for v0.1?

**Answer:** That first one and the validate would be nice

---

### 9. Testing Strategy
**Question:** How should we approach testing?
- Unit tests for each component?
- Integration tests with sample format definitions?
- Golden file testing for code generation?
- Should we create a test suite of common formats (PNG, ZIP, etc.)?

**Answer:** Unit tests and integration tests for a few common file formats

---

### 10. Dependencies
**Question:** Which Rust dependencies should we use?
- Parser: pest, nom, lalrpop, chumsky, hand-written?
- WASM runtime: wasmtime, wasmer?
- CLI framework: clap?
- Other essential crates?

**Answer:** For parser we should strive to use a library, whatever one you know best for our use case. Same for wasm runtime, though I believe I have heard more about wasmtime. Clap is fine. In general, use whatever crates you deem necessary to prevent us needing to create as much boilerplate or reinvent the wheel

---

### 11. Project Structure
**Question:** How should we organize the codebase?
- Monorepo with workspace members (dezzy-core, dezzy-cli, dezzy-backend-cpp, etc.)?
- Single crate initially, split later?
- Where should backends live?

**Answer:** Monorepo

---

### 12. Endianness Handling
**Question:** How should dezzy handle different byte orders?
- Explicit endianness annotations in DSL?
- Default endianness per format?
- Runtime vs compile-time handling?

**Answer:** Explicit annotations in DSL

---

## Consequences

### Technical Decisions
1. **YAML-based DSL** - Familiar syntax for users coming from Kaitai Struct, easy to parse with serde_yaml
2. **Multi-stage IR** - Allows for optimization passes and clean separation of concerns (parsing → high-level IR → low-level IR → codegen)
3. **Simple expressions** - Reduces complexity in phase 1, can be extended later
4. **WASM-only plugins** - Simplifies plugin system, wasmtime provides sandboxing and cross-platform support
5. **C++17 header-only with STL** - Modern, safe, easy to integrate into user projects
6. **Ariadne for errors** - Beautiful, user-friendly error diagnostics from day one
7. **Monorepo structure** - Easier to maintain consistency across components

### Implementation Plan
**Workspace Structure:**
- `dezzy-core`: IR types, multi-stage pipeline, core logic
- `dezzy-parser`: YAML DSL parsing and validation
- `dezzy-backend`: Backend abstraction, WASM plugin system
- `dezzy-backend-cpp`: C++ code generator (native implementation)
- `dezzy-cli`: CLI tool with compile and validate commands

**Key Dependencies:**
- `serde` + `serde_yaml`: YAML DSL parsing
- `wasmtime`: WASM runtime for plugins
- `clap`: CLI framework
- `ariadne`: Error reporting
- `anyhow`: Error handling
- `thiserror`: Custom error types

**Phase 1 Scope:**
Focus on basic type definitions (integers with endianness, fixed-size arrays, structs) with read & write support. Deferred features: conditionals, computed fields, enums, validation.

## Notes
ADR approved and implementation plan created. See docs/architecture.md for detailed design.

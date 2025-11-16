# Dezzy Project Tasks

## Current Phase
Phase 1: Core Infrastructure - COMPLETE

## Completed Tasks

### Phase 0: Architecture & Design
- [x] Complete ADR-001 (Architecture Design Record)
- [x] Document project structure (architecture.md)
- [x] Create initial documentation

### Phase 1: Core Infrastructure
- [x] Set up Cargo workspace structure
- [x] Implement YAML DSL parser
- [x] Design and implement multi-stage IR (HIR + LIR)
- [x] Implement pipeline for HIR → LIR transformation
- [x] Basic error handling with thiserror

### Phase 1.5: Backend System
- [x] Design backend trait API
- [x] Create plugin registry
- [x] WASM backend stub (for future implementation)

### Phase 1.6: C++ Backend
- [x] Implement C++ code generator
- [x] Generate header-only C++17 code
- [x] Support for primitive types (u8-u64, i8-i64)
- [x] Support for fixed-size arrays
- [x] Read and write operations with endianness support
- [x] Reader/Writer helper classes

### Phase 1.7: CLI
- [x] Implement compile command
- [x] Implement validate command
- [x] File I/O and error reporting
- [x] Example format (simple.yaml)

## Known Issues / TODOs

### High Priority
- [ ] Use actual field names in generated C++ code (currently using field_0, field_1, etc.)
- [ ] Preserve endianness from YAML format to LIR (currently hardcoded)
- [ ] Add comprehensive unit tests for each crate
- [ ] Add integration tests with real formats
- [ ] Improve error messages with ariadne (currently basic)

### Medium Priority
- [ ] Support for struct nesting in generated code
- [ ] Add documentation comments to generated C++ code
- [ ] Create more example formats
- [ ] Add README with usage examples
- [ ] Set up CI/CD pipeline

### Low Priority / Future
- [ ] Implement WASM plugin loading
- [ ] Support for conditional parsing
- [ ] Support for computed fields
- [ ] Support for enums
- [ ] Support for validation/assertions
- [ ] Additional backends (Python, Rust, etc.)
- [ ] Format validation (type checking, circular references)
- [ ] Code formatter for YAML DSL

## Notes
Phase 1 is complete with basic functionality working end-to-end. The system can parse YAML format definitions, lower them to IR, and generate working C++17 code.

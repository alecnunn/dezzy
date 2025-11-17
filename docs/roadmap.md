# Dezzy Development Roadmap

This document outlines the development roadmap for dezzy, a declarative binary format parser generator.

## Project Vision

Dezzy aims to make binary format parsing and generation as simple as writing a YAML schema. Key goals:
- **Declarative**: Describe the format, not the parsing logic
- **Type-safe**: Generate strongly-typed code in multiple languages
- **Complete**: Support real-world formats with all their complexity
- **Extensible**: Plugin system for custom backends and transformations

## ✅ Completed Phases

### Phase 1-3: Array Types (Complete)
**Status**: ✅ Shipped

**Features**:
- Count-prefixed arrays: `type: u32[count]`
- Until-EOF arrays: `type: u8[]`
- Until-condition arrays: `type: Chunk[]` with `until: chunk_type equals 'IEND'`

**Commit**: a2bea7f, 2b766ba, 16d2345

---

### Phase 4: Enums (Complete)
**Status**: ✅ Shipped

**Features**:
- Enum definitions with explicit values
- Type-safe enum generation in C++ (`enum class`) and Python (`IntEnum`)
- Enum field usage with proper type checking

**Commit**: e70aecc

---

### Phase 5: Assertions and Validation (Complete)
**Status**: ✅ Shipped

**Features**:
- Field assertions: `assert: { equals: 0x04034b50 }`
- Runtime validation with descriptive errors
- Supports equality checks on primitives and byte arrays

**Commit**: dc4ab21, 0582819

---

### Phase 6: String Types (Complete)
**Status**: ✅ Shipped

**Features**:
- Fixed-length strings: `type: str[16]`
- Null-terminated strings: `type: cstr`
- Length-prefixed strings: `type: str(length_field)`
- UTF-8 encoding/decoding in generated code

**Commit**: bd118d9, e3cf213

---

### Phase 7.1: Blob Type and Skip Directives (Complete)
**Status**: ✅ Shipped

**Features**:
- Blob type for raw byte data: `type: blob(size_field)`
- Skip directive for ignoring bytes: `skip: fixed(8)`
- Variable-length skip based on field values

**Commit**: d670a59, 6b213ed

---

### Phase 7.2/7.3: Padding, Alignment, and Bitfields (Complete)
**Status**: ✅ Shipped

**Features**:
- Fixed padding: `padding: 4`
- Alignment: `align: 8`
- Bitfields: `type: u3`, `type: i5` (1-7 bit signed/unsigned)
- BitReader/BitWriter with MSB-first bit ordering

**Commit**: 2755dd5, a78f3ef, 9e0f88b

---

### Phase 8: Real-World Format Validation (Complete)
**Status**: ✅ Shipped

**Features**:
- PNG format: 72KB file, 12 chunks, all parsed successfully
- ZIP format: 395 bytes, complete directory structure
- Validation of all core features with real files

**Commit**: be1a44e

**Documentation**: `docs/real-world-formats.md`

---

### Phase 9: Conditional Parsing (Complete)
**Status**: ✅ Shipped

**Features**:
- Conditional fields: `if: version equals 1`
- Optional types in C++ (`std::optional<T>`) and Python (`Optional[T]`)
- Runtime condition evaluation
- Support for comparison operators

**Commits**: 5ecb491, bdb6a18

**Documentation**:
- `docs/phase-9-conditional-parsing.md`
- `docs/phase-9-known-limitations.md`

**Known Limitations**:
- Optional array initialization needs improvement
- Optional field writes need `.value()` extraction
- Complex nested conditions not yet supported

---

## 🚧 Planned Phases

### Phase 10: Computed/Derived Fields
**Status**: 📋 Planned (Next)

**Priority**: HIGH - Addresses Phase 9 limitations and enables proper write operations

**Goal**: Enable fields whose values are automatically calculated from other fields.

**Use Cases**:
- Auto-calculate array lengths on write
- Compute offsets and file positions
- Derive checksums and totals
- Calculate padding amounts

**Proposed Syntax**:
```yaml
- name: entries
  type: Entry[]

- name: num_entries
  type: u32
  computed: entries.length  # Auto-filled on write

- name: header_size
  type: u16
  computed: 14  # Constant value

- name: file_offset
  type: u32
  computed: current_position + header_size
```

**Implementation Considerations**:
- Read mode: Validate computed values match actual values (optional)
- Write mode: Automatically fill in computed values
- Dependency ordering: Ensure fields are computed in correct order
- Expression evaluation: Reuse existing expression infrastructure

**Backends**:
- C++: Generate computation in write() method
- Python: Generate computation in write() method

**Success Criteria**:
- ✓ Auto-calculate array lengths
- ✓ Compute field-based expressions
- ✓ Support constants
- ✓ Validate on read (optional)
- ✓ Auto-fill on write

---

### Phase 11: Checksums and Integrity Validation
**Status**: 📋 Planned

**Priority**: HIGH - Critical for many binary formats

**Goal**: Built-in support for checksums, CRCs, and hash functions.

**Use Cases**:
- PNG chunk CRCs
- ZIP file CRC32
- Binary protocol checksums
- Data integrity validation

**Proposed Syntax**:
```yaml
- name: chunk_data
  type: u8[chunk_length]

- name: chunk_crc
  type: u32
  checksum:
    algorithm: crc32
    over: [chunk_type, chunk_data]

- name: header_checksum
  type: u16
  checksum:
    algorithm: crc16-ccitt
    over: { start: 0, end: header_size }
```

**Supported Algorithms**:
- CRC32, CRC32C
- CRC16, CRC16-CCITT
- Adler32
- MD5, SHA1, SHA256 (for non-performance-critical use)

**Implementation Considerations**:
- Read mode: Validate checksum matches
- Write mode: Automatically compute and write checksum
- Range specification: Support byte ranges, field lists
- Algorithm plugins: Allow custom checksum implementations

**Backends**:
- C++: Use standard library or lightweight CRC implementations
- Python: Use `zlib.crc32()`, `hashlib`

---

### Phase 12: Union/Variant Types
**Status**: 📋 Planned

**Priority**: MEDIUM - Improves type safety for variant data

**Goal**: Tagged unions for mutually exclusive fields based on discriminator.

**Use Cases**:
- Protocol message types
- Variant record formats
- Type-dependent data structures

**Proposed Syntax**:
```yaml
- name: message_type
  type: u8

- name: message_payload
  type: union
  tag: message_type
  variants:
    0x01:
      name: TextMessage
      type: TextMessageData
    0x02:
      name: BinaryMessage
      type: BinaryMessageData
    0x03:
      name: StatusMessage
      type: StatusMessageData
```

**Implementation Considerations**:
- C++: `std::variant<TextMessageData, BinaryMessageData, StatusMessageData>`
- Python: Union types with type guards
- Read mode: Parse based on tag value
- Write mode: Select variant based on discriminator
- Type safety: Ensure only one variant is active

**Alternative Syntax** (inline variants):
```yaml
- name: payload
  type: union
  tag: message_type
  cases:
    1:
      - name: text_length
        type: u16
      - name: text_data
        type: u8[text_length]
    2:
      - name: binary_size
        type: u32
      - name: binary_data
        type: u8[binary_size]
```

---

### Phase 13: Compression Support
**Status**: 💭 Future

**Priority**: MEDIUM - Useful for many formats

**Goal**: Transparent compression/decompression of data fields.

**Use Cases**:
- ZIP compressed data
- PNG IDAT chunks (zlib)
- Protocol buffer compression
- Any format with inline compression

**Proposed Syntax**:
```yaml
- name: compressed_size
  type: u32

- name: uncompressed_size
  type: u32

- name: compressed_data
  type: u8[compressed_size]
  compression:
    algorithm: zlib
    uncompressed_size: uncompressed_size
```

**Supported Algorithms**:
- zlib/deflate
- gzip
- lz4
- zstd
- bzip2

**Implementation Considerations**:
- Read mode: Decompress automatically, expose decompressed data
- Write mode: Compress data, update size fields
- Streaming: Support incremental decompression for large data
- Error handling: Handle compression errors gracefully

---

### Phase 14: Custom Transformations
**Status**: 💭 Future

**Priority**: LOW - Advanced feature

**Goal**: Apply custom transformations to field data.

**Use Cases**:
- Encryption/decryption
- Base64 encoding/decoding
- Custom encoding schemes
- Data obfuscation

**Proposed Syntax**:
```yaml
- name: encrypted_data
  type: u8[data_length]
  transform:
    type: xor
    key: encryption_key

- name: encoded_string
  type: str(32)
  transform:
    type: base64
```

---

### Phase 15: Incremental/Streaming Parsing
**Status**: 💭 Future

**Priority**: MEDIUM - Enables large file support

**Goal**: Parse files incrementally without loading entire file into memory.

**Use Cases**:
- Very large files (GB+)
- Network streams
- Embedded systems with limited memory

**Implementation Considerations**:
- Callback-based parsing
- Lazy field evaluation
- Buffered reading
- Seek-free streaming where possible

---

### Phase 16: Backward Compatibility and Versioning
**Status**: 💭 Future

**Priority**: LOW - Nice to have

**Goal**: Handle format version migrations and backward compatibility.

**Proposed Syntax**:
```yaml
versions:
  - version: 1
    fields:
      - name: legacy_field
        type: u32

  - version: 2
    fields:
      - name: legacy_field
        type: u32
      - name: new_field
        type: u64

migration:
  from: 1
  to: 2
  transform:
    new_field: 0  # Default value
```

---

## 🎯 Immediate Priorities

### Phase 10 (Next Sprint)
**Computed/Derived Fields** - Highest value, addresses current limitations

**Tasks**:
1. Design expression evaluation for computed values
2. Implement read-time validation (optional)
3. Implement write-time auto-fill
4. Add dependency ordering for computed fields
5. Test with PNG/ZIP formats (auto-calculate lengths)

**Estimated Effort**: 2-3 days

---

### Technical Debt and Improvements

**Known Issues to Address**:
1. **Phase 9 Limitations** (see `docs/phase-9-known-limitations.md`):
   - Optional array initialization (C++)
   - Optional field writes need `.value()` extraction
   - None checks in Python writes

2. **Code Quality**:
   - Reduce code duplication in backends
   - Improve error messages
   - Add more comprehensive testing

3. **Performance**:
   - Optimize expression evaluation
   - Reduce allocations in hot paths
   - Profile real-world format parsing

4. **Documentation**:
   - Tutorial for creating new formats
   - Backend development guide
   - Expression syntax reference

---

## 📊 Feature Matrix

| Feature | Phase | C++ | Python | Tested |
|---------|-------|-----|--------|--------|
| Primitive types | 1 | ✅ | ✅ | ✅ |
| Fixed arrays | 1 | ✅ | ✅ | ✅ |
| Dynamic arrays | 1 | ✅ | ✅ | ✅ |
| Until-EOF arrays | 2 | ✅ | ✅ | ✅ |
| Until-condition arrays | 3 | ✅ | ✅ | ✅ |
| Enums | 4 | ✅ | ✅ | ✅ |
| Assertions | 5 | ✅ | ✅ | ✅ |
| Strings | 6 | ✅ | ✅ | ✅ |
| Blobs | 7.1 | ✅ | ✅ | ✅ |
| Skip directives | 7.1 | ✅ | ✅ | ✅ |
| Padding/Alignment | 7.2 | ✅ | ✅ | ✅ |
| Bitfields | 7.3 | ✅ | ✅ | ✅ |
| Conditional fields | 9 | ✅⚠️ | ✅⚠️ | ✅ |
| Computed fields | 10 | 📋 | 📋 | - |
| Checksums | 11 | 📋 | 📋 | - |
| Unions | 12 | 📋 | 📋 | - |
| Compression | 13 | 💭 | 💭 | - |

Legend: ✅ Complete | ✅⚠️ Complete with limitations | 📋 Planned | 💭 Future

---

## 🏗️ Architecture Evolution

### Current Architecture (Phase 9)
```
YAML Schema → Parser → HIR → Pipeline → LIR → Backends → Code
                                                ├─ C++ (built-in)
                                                └─ Python (WASM plugin)
```

### Planned Enhancements
- **Plugin API v2**: More backend hooks for custom behavior
- **LIR Optimization**: Deduplicate operations, constant folding
- **Multi-pass compilation**: Enable forward references, complex dependencies
- **IR validation**: Catch errors earlier in pipeline

---

## 📈 Success Metrics

### Phase Completion Criteria
Each phase must achieve:
- ✓ Design document with examples
- ✓ Implementation in core (HIR/LIR)
- ✓ C++ backend support
- ✓ Python backend support
- ✓ Test examples demonstrating feature
- ✓ Real-world format validation (if applicable)
- ✓ Documentation of known limitations

### Project Success Metrics
- Parse 10+ real-world binary formats successfully
- Generated code is idiomatic and performant
- Plugin system supports 3+ backends
- Comprehensive test coverage (>80%)
- Active community usage

---

## 🤝 Contributing

Interested in contributing? Priority areas:
1. **New backends**: Go, Rust, TypeScript, C#
2. **Format libraries**: Create YAML definitions for common formats
3. **Testing**: More real-world format validation
4. **Documentation**: Tutorials, guides, examples

---

## 📚 References

- **Design Documents**: `docs/phase-*.md`
- **Architecture**: `docs/architecture.md`, `docs/ADR-*.md`
- **Examples**: `examples/*.yaml`
- **Real-world Formats**: `docs/real-world-formats.md`

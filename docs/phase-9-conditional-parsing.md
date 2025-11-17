# Phase 9: Conditional Parsing Design

## Overview
Enable fields that only exist when certain conditions are met, allowing formats with optional/variant fields based on context.

## Motivation

Many binary formats have optional fields:
- **Compression**: Data field only present if `compression_method != 0`
- **Versioning**: Extended headers in newer versions
- **Flags**: Features enabled/disabled by bit flags
- **Type variants**: Different fields for different message types

### Example Use Cases

**PNG tRNS chunk** - Transparency data varies by color type:
```yaml
- name: TRNS
  fields:
    - name: color_type
      type: u8
    - name: gray_alpha
      type: u16
      if: color_type == 0  # Grayscale
    - name: rgb_alpha
      type: u16[3]
      if: color_type == 2  # RGB
```

**ZIP with encryption**:
```yaml
- name: LocalFileHeader
  fields:
    - name: flags
      type: u16
    - name: encryption_header
      type: u8[12]
      if: (flags & 0x01) != 0  # Bit 0 = encrypted
```

**Protocol versioning**:
```yaml
- name: Message
  fields:
    - name: version
      type: u8
    - name: legacy_data
      type: u32
      if: version < 2
    - name: extended_data
      type: ExtendedHeader
      if: version >= 2
```

## Proposed Syntax

### YAML DSL
```yaml
fields:
  - name: field_name
    type: field_type
    if: condition_expression  # Optional condition
```

### Condition Expressions
Reuse existing expression syntax from `until` conditions:
- Comparisons: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Bitwise: `&`, `|`, `^`, `<<`, `>>`
- Logical: `&&`, `||`, `!`
- Parentheses for grouping
- Array indexing: `array[-1]` (for until conditions)

## Type System Changes

### HIR (High-Level IR)
```rust
struct HirField {
    name: String,
    field_type: HirType,
    assertion: Option<HirAssertion>,
    skip: Option<Skip>,
    if_condition: Option<Expr>,  // NEW
}
```

### Field Optionality
Conditional fields become optional in the generated struct:
```cpp
struct Message {
    uint8_t version;
    std::optional<uint32_t> legacy_data;      // if: version < 2
    std::optional<ExtendedHeader> extended_data;  // if: version >= 2
};
```

## LIR Operations

### New Operations
```rust
enum LirOperation {
    // Existing operations...

    ConditionalBlock {
        condition: Expr,
        true_ops: Vec<LirOperation>,
        false_ops: Vec<LirOperation>,  // Optional else branch
    },
}
```

### Alternative: Reuse Existing Operations
Instead of a new operation, emit existing operations with conditional wrapping:
```rust
// In pipeline lowering:
if field.if_condition.is_some() {
    // Wrap read/write ops in conditional check
    operations.push(check_condition(field.if_condition));
    operations.push(conditional_jump_if_false(skip_label));
    operations.push(read_operation);
    operations.push(label(skip_label));
}
```

## Code Generation

### C++ Backend
```cpp
// Conditional field reading
struct Message {
    uint8_t version;
    std::optional<uint32_t> legacy_data;

    static Message read(Reader& reader) {
        Message result;
        result.version = reader.read_le<uint8_t>();

        // Conditional read
        if (result.version < 2) {
            result.legacy_data = reader.read_le<uint32_t>();
        }

        return result;
    }

    void write(Writer& writer) const {
        writer.write_le(version);

        // Conditional write
        if (version < 2) {
            if (legacy_data.has_value()) {
                writer.write_le(legacy_data.value());
            } else {
                throw std::runtime_error("legacy_data required when version < 2");
            }
        }
    }
};
```

### Python Backend
```python
@dataclass
class Message:
    version: int
    legacy_data: Optional[int] = None

    @staticmethod
    def read(buffer: bytes, offset: int = 0):
        pos = offset
        version = struct.unpack_from('B', buffer, pos)[0]
        pos += 1

        legacy_data = None
        if version < 2:
            legacy_data = struct.unpack_from('<I', buffer, pos)[0]
            pos += 4

        return Message(version, legacy_data), pos - offset

    def write(self) -> bytes:
        result = bytearray()
        result.extend(struct.pack('B', self.version))

        if self.version < 2:
            if self.legacy_data is not None:
                result.extend(struct.pack('<I', self.legacy_data))
            else:
                raise ValueError("legacy_data required when version < 2")

        return bytes(result)
```

## Implementation Plan

### 1. Schema Changes
- Add `if: <expr>` field to YamlField
- Parse condition expression (reuse `parse_condition` from until-conditions)

### 2. HIR Changes
- Add `if_condition: Option<Expr>` to HirField
- Validate that condition references valid fields (must be defined before conditional field)

### 3. Pipeline Changes
- When lowering conditional field:
  - Evaluate condition in context
  - Wrap read/write operations in conditional block
  - Mark field as optional in LIR type info

### 4. LIR Changes (Option A - Simple)
Add metadata to LirField:
```rust
struct LirField {
    name: String,
    type_info: String,
    is_optional: bool,  // NEW
    condition: Option<Expr>,  // NEW
}
```

Then backends generate conditionals when `condition.is_some()`.

### 5. Backend Changes
- C++: Use `std::optional<T>` for conditional fields
- Python: Use `Optional[T]` type hint, default to None
- Generate if-statement around read/write for conditional fields
- Reuse expression code generation from until-conditions

### 6. Testing
Create test formats:
- Simple flag-based conditional
- Version-based conditional
- Bitwise flag conditional
- Multiple conditionals in same struct

## Edge Cases & Considerations

### Validation
**Problem**: What if write-time condition differs from read-time?
```cpp
Message m;
m.version = 1;  // legacy_data required
m.legacy_data = std::nullopt;  // But not provided!
m.write();  // Should this error?
```

**Solution**: Validate at write time that required fields are present.

### Circular Dependencies
**Problem**: Field condition references field that comes after it.
```yaml
- name: data
  if: enable_data == 1
- name: enable_data  # ERROR: used before defined
  type: u8
```

**Solution**: Only allow conditions to reference fields defined earlier.

### Complex Conditions
**Problem**: Condition references array elements or nested fields.
```yaml
- name: options
  if: header.flags[2] == 1  # Nested access
```

**Solution**: Phase 9 supports simple field references only. Defer complex expressions to later phase.

### Multiple Alternatives
**Problem**: Mutually exclusive fields.
```yaml
- name: old_format
  if: version == 1
- name: new_format
  if: version == 2
```

**Solution**: Both fields optional, at most one has value. Union types (Phase 12) provide better solution.

## Example: Extended ZIP Local File Header

Real-world example - ZIP64 extended information:

```yaml
- name: LocalFileHeader
  fields:
    - name: signature
      type: u32
      assert: { equals: 0x04034b50 }
    - name: version_needed
      type: u16
    - name: flags
      type: u16
    - name: compression_method
      type: u16
    - name: compressed_size
      type: u32
    - name: filename_length
      type: u16
    - name: extra_field_length
      type: u16
    - name: filename
      type: u8[filename_length]
    - name: extra_field
      type: u8[extra_field_length]

    # Actual compressed data - only if compression used
    - name: compressed_data
      type: u8[compressed_size]
      if: compression_method != 0

    # Data descriptor - only if bit 3 of flags is set
    - name: data_descriptor
      type: DataDescriptor
      if: (flags & 0x08) != 0
```

## Success Criteria

1. ✓ Simple conditionals compile and generate correct code
2. ✓ C++ uses `std::optional<T>` for conditional fields
3. ✓ Python uses `Optional[T]` for conditional fields
4. ✓ Read operations skip field when condition false
5. ✓ Write operations validate required conditional fields present
6. ✓ Conditions can reference any previously-defined field
7. ✓ Expression evaluation reuses existing expr infrastructure
8. ✓ Test with real format (ZIP data descriptor, PNG variant chunks)

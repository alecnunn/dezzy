# Phase 9: Conditional Parsing - Known Limitations

This document outlines the known limitations of the conditional parsing implementation in Phase 9.

## 1. Optional Array Initialization (C++)

**Issue**: When reading conditional array fields, the C++ backend doesn't initialize the optional before accessing array elements.

**Example**:
```yaml
- name: compressed_data
  type: u8[4]
  if: compression_method not-equals 0
```

**Generated Code** (incorrect):
```cpp
if ((result.compression_method != 0)) {
    for (size_t i = 0; i < 4; ++i) {
        result.compressed_data[i] = reader.read_le<uint8_t>();  // ERROR: optional not initialized
    }
}
```

**Expected Code**:
```cpp
if ((result.compression_method != 0)) {
    result.compressed_data = std::array<uint8_t, 4>();  // Initialize the optional
    for (size_t i = 0; i < 4; ++i) {
        (*result.compressed_data)[i] = reader.read_le<uint8_t>();
    }
}
```

**Workaround**: Use scalar types for conditional fields, not arrays.

**Fix Required**: Detect array types in conditional fields and generate proper optional initialization and dereferencing code.

---

## 2. Optional Field Writing (C++)

**Issue**: When writing conditional fields, the C++ backend doesn't extract the value from the optional before writing.

**Example**:
```yaml
- name: v1_data
  type: u32
  if: version equals 1
```

**Generated Code** (incorrect):
```cpp
if ((version == 1)) {
    writer.write_le(v1_data);  // ERROR: trying to write std::optional<uint32_t>
}
```

**Expected Code**:
```cpp
if ((version == 1)) {
    if (v1_data.has_value()) {
        writer.write_le(v1_data.value());
    } else {
        throw std::runtime_error("v1_data required when version == 1");
    }
}
```

**Workaround**: None - this will cause compilation errors. The generated C++ code needs manual fixing.

**Fix Required**:
- Detect when a field being written is optional (check `is_optional` flag)
- Generate `.value()` extraction or proper error handling
- Add runtime validation that required conditional fields are present

---

## 3. Optional Field Writing (Python)

**Issue**: Similar to C++, Python write operations don't handle None values for conditional fields.

**Example**:
```yaml
- name: extra_info
  type: u16
  if: flags greater-than 0
```

**Generated Code** (incorrect):
```python
if (self.flags > 0):
    result.extend(struct.pack('<H', self.extra_info))  # ERROR: might be None
```

**Expected Code**:
```python
if (self.flags > 0):
    if self.extra_info is not None:
        result.extend(struct.pack('<H', self.extra_info))
    else:
        raise ValueError("extra_info required when flags > 0")
```

**Workaround**: Ensure fields are set before calling write(), or the code will raise exceptions at runtime.

**Fix Required**:
- Check `is_optional` flag during write code generation
- Add None checks and raise ValueError if required field is missing

---

## 4. Nested Field Access in Conditions

**Issue**: Conditions can only reference simple field names, not nested field access or array indexing.

**Example** (not supported):
```yaml
- name: extended_header
  type: ExtendedHeader
  if: header.flags[2] equals 1  # ERROR: nested access not supported
```

**Current Support**: Only simple field references like `version equals 1` or `flags greater-than 0`.

**Workaround**: Restructure format to use flat field references.

**Fix Required**: Extend expression parser and codegen to handle:
- Field access: `header.flags`
- Array indexing: `flags[2]`
- Bitwise operations: `(flags & 0x04) != 0`

---

## 5. Complex Conditional Expressions

**Issue**: Limited support for complex boolean expressions.

**Current Support**:
- Simple comparisons: `version equals 1`
- Basic operators: `==`, `!=`, `<`, `>`, `<=`, `>=`

**Not Yet Supported**:
- Bitwise operations in conditions: `(flags & 0x08) != 0`
- Logical operators: `AND`, `OR`
- Arithmetic in conditions: `version + offset >= 5`

**Workaround**: Use simple field comparisons only.

**Fix Required**: The expression parser already supports these, but they haven't been thoroughly tested with conditional fields.

---

## 6. Mutually Exclusive Fields

**Issue**: No way to express that exactly one of multiple fields should be present (union/variant types).

**Example**:
```yaml
- name: v1_format
  type: V1Data
  if: version equals 1

- name: v2_format
  type: V2Data
  if: version equals 2
```

Both fields are `Optional`, but semantically exactly one should have a value. This isn't enforced.

**Workaround**: Use conditional checks in application logic.

**Fix Required**: Design and implement union/variant types (future phase - Phase 12).

---

## Summary

Most limitations relate to edge cases in type handling (arrays, nested access) and validation (ensuring required fields are present). The core conditional parsing infrastructure works correctly for simple scalar fields.

**Priority fixes**:
1. Optional field writing with .value() extraction (both C++ and Python)
2. Optional array initialization (C++)
3. Runtime validation of required conditional fields

**Future enhancements**:
4. Nested field access in conditions
5. Union/variant types for mutually exclusive fields

# PNG and ZIP Format Requirements Analysis

## PNG Format Structure

### File Header
```
- Signature: 8 bytes (fixed: 0x89, 'P', 'N', 'G', '\r', '\n', 0x1a, '\n')
```

### Chunk Structure (Repeating)
```
- Length: u32 (number of bytes in chunk data)
- Type: 4 bytes ASCII (e.g., "IHDR", "IDAT", "IEND")
- Data: byte array[Length]
- CRC: u32 (CRC-32 of Type + Data)
```

### Critical Features Needed
1. ✅ Fixed-size byte arrays (signature)
2. ❌ **Variable-length arrays** (data sized by Length field)
3. ❌ **Repeating structures** (multiple chunks until IEND)
4. ❌ String/ASCII types (chunk type)
5. ⚠️ Computed fields (CRC - can skip validation for now)

## ZIP Format Structure

### Local File Header
```
- Signature: u32 (0x04034b50)
- Version needed: u16
- Flags: u16
- Compression method: u16
- Last mod time: u16
- Last mod date: u16
- CRC-32: u32
- Compressed size: u32
- Uncompressed size: u32
- File name length: u16
- Extra field length: u16
- File name: byte array[File name length]
- Extra field: byte array[Extra field length]
- Compressed data: byte array[Compressed size]
```

### Central Directory Header
(Similar structure, also needs variable-length arrays)

### End of Central Directory Record
```
- Signature: u32
- Disk number: u16
- ... more fields ...
- Comment length: u16
- Comment: byte array[Comment length]
```

### Critical Features Needed
1. ✅ All primitive integer types
2. ❌ **Variable-length arrays** (file name, extra field, data)
3. ❌ **Repeating structures** (multiple file entries)
4. ❌ String types (file names)
5. ⚠️ Seeking/offsets (can be emulated with skipping)

## Priority Implementation Order

### Phase 1: Variable-Length Arrays (CRITICAL)
Enable: `data: u8[length_field]` syntax

**DSL Syntax:**
```yaml
fields:
  - name: data_length
    type: u32
  - name: data
    type: u8[data_length]  # Size from previous field
```

**IR Changes:**
- HIR: Add `HirType::DynamicArray { element_type, size_field }`
- LIR: Add operations for reading/writing dynamic arrays

**C++ Generation:**
- Use `std::vector<T>` for dynamic arrays
- Read size field, then read that many elements

### Phase 2: Repeating Structures
Enable: Arrays of structs with dynamic count

**DSL Syntax:**
```yaml
fields:
  - name: chunk_count
    type: u32
  - name: chunks
    type: Chunk[chunk_count]
```

Or unlimited until condition:
```yaml
fields:
  - name: chunks
    type: Chunk[]
    until: chunk.type == "IEND"  # Future: conditional expressions
```

### Phase 3: String Types
Basic string support for file names, ASCII chunk types

**DSL Syntax:**
```yaml
fields:
  - name: file_name
    type: string[name_length]  # Fixed-length string
```

## Simplified Initial Target: PNG Chunk

Can we parse a single PNG chunk with just variable-length arrays?

```yaml
name: PNGChunk
endianness: big

types:
  - name: Chunk
    type: struct
    fields:
      - name: length
        type: u32
      - name: chunk_type
        type: u8[4]  # Fixed size (type code)
      - name: data
        type: u8[length]  # VARIABLE SIZE - key feature!
      - name: crc
        type: u32
```

This requires just **variable-length arrays** - our highest priority feature!

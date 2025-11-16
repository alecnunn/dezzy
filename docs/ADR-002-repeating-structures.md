# ADR-002: Repeating Structures for Binary Formats

## Status
Proposed

## Context
Dezzy needs to support repeating structures to handle real-world binary formats including:
- **File formats**: PNG (multiple chunks), ZIP (multiple entries), ELF (multiple sections)
- **Network protocols**: HTTP (multiple headers), DNS (multiple records), TCP segments
- **Binary serialization**: Protocol Buffers (repeated fields), MessagePack (arrays)
- **Database formats**: Multiple records, log entries, indices
- **Container formats**: TAR archives, video frames, audio samples

Currently, we only support single instances of structures. This limits us to parsing individual records, not complete formats.

## Decision

### Syntax Patterns

Support four primary patterns for repeating structures:

#### 1. Count-Prefixed Arrays (Most Common)
```yaml
fields:
  - name: num_items
    type: u32
  - name: items
    type: Item[num_items]
```

**Use cases**: ZIP entries, ELF sections, most binary formats

#### 2. Until Condition
```yaml
fields:
  - name: chunks
    type: Chunk[]
    repeat-until: chunks[-1].chunk_type == [73, 69, 78, 68]  # "IEND"
```

**Use cases**: PNG chunks, protocol messages with terminators, linked lists

#### 3. Until EOF
```yaml
fields:
  - name: records
    type: Record[]
    repeat-until: eof
```

**Use cases**: Log files, simple record streams, CSV-like binary formats

#### 4. While Condition (Optional - Future)
```yaml
fields:
  - name: packets
    type: Packet[]
    repeat-while: packets[-1].has_more_flag
```

**Use cases**: Fragmented packets, continuation flags

### HIR Representation

Extend `HirType` with:
```rust
enum HirType {
    // ... existing types ...

    /// Array of structs with size from field
    StructArray {
        element_type: String,  // Struct name
        count_field: String,   // Field containing count
    },

    /// Array repeated until condition (future)
    RepeatUntil {
        element_type: String,
        condition: RepeatCondition,
    },
}

enum RepeatCondition {
    Eof,
    Expression(String),  // e.g., "item.type == END"
}
```

### LIR Representation

```rust
enum LirOperation {
    // ... existing operations ...

    /// Read array of structs
    ReadStructArray {
        dest: VarId,
        element_type: String,
        count_var: VarId,  // Variable containing count
    },

    /// Write array of structs
    WriteStructArray {
        src: VarId,
        element_type: String,
        size_field_name: String,
    },
}
```

### Code Generation

**C++:**
```cpp
std::vector<Chunk> chunks;
chunks.reserve(chunk_count);
for (uint32_t i = 0; i < chunk_count; ++i) {
    chunks.push_back(Chunk::read(reader));
}
```

**Python:**
```python
chunks = []
for _ in range(chunk_count):
    chunk, bytes_read = Chunk.read(buffer, pos)
    pos += bytes_read
    chunks.append(chunk)
```

### Examples

**PNG File (Multiple Chunks):**
```yaml
name: PNG
endianness: big

types:
  - name: Chunk
    type: struct
    fields:
      - name: length
        type: u32
      - name: chunk_type
        type: u8[4]
      - name: data
        type: u8[length]
      - name: crc
        type: u32

  - name: PNGFile
    type: struct
    fields:
      - name: signature
        type: u8[8]
      - name: chunks
        type: Chunk[]
        repeat-until: chunks[-1].chunk_type == [73, 69, 78, 68]  # "IEND"
```

**Network Packet (Count-Prefixed):**
```yaml
name: NetworkPacket
endianness: little

types:
  - name: Header
    type: struct
    fields:
      - name: name_length
        type: u8
      - name: name
        type: u8[name_length]
      - name: value_length
        type: u16
      - name: value
        type: u8[value_length]

  - name: Packet
    type: struct
    fields:
      - name: header_count
        type: u16
      - name: headers
        type: Header[header_count]
      - name: payload_length
        type: u32
      - name: payload
        type: u8[payload_length]
```

**Log File (Until EOF):**
```yaml
name: BinaryLog
endianness: native

types:
  - name: LogEntry
    type: struct
    fields:
      - name: timestamp
        type: u64
      - name: level
        type: u8
      - name: message_length
        type: u16
      - name: message
        type: u8[message_length]

  - name: LogFile
    type: struct
    fields:
      - name: entries
        type: LogEntry[]
        repeat-until: eof
```

## Implementation Plan

### Phase 1: Count-Prefixed Arrays (This PR)
- ✅ Most common pattern
- ✅ Works with existing field tracking
- ✅ No complex expression parsing needed
- ✅ Enables: Complete PNG files, ZIP archives, most protocols

### Phase 2: Until EOF (Next PR)
- Simple to implement (just check reader.remaining() == 0)
- Enables: Log files, simple record streams

### Phase 3: Until Condition (Future PR)
- Requires expression parser/evaluator
- Complex but powerful
- Enables: PNG chunks without count, protocol state machines

## Consequences

### Positive
- Handles real-world formats with multiple records
- Syntax mirrors actual binary format structure
- Progressive enhancement (start simple, add complexity)
- Same pattern works for files, protocols, streams

### Negative
- Expressions add complexity (Phase 3)
- Need to track index for repeat-until conditions
- Performance considerations for large arrays

### Neutral
- C++ uses std::vector (already used for dynamic arrays)
- Python uses list comprehensions (natural fit)
- No new dependencies required

## User Decisions

1. **Initial scope**: ✅ Count-prefixed arrays only (Phase 1)
2. **Syntax preference**: ✅ `until` (shorter, cleaner)
3. **Expression syntax**: ✅ Custom DSL (see below)
4. **Array bounds**: Deferred to Phase 2 (add `max-count` if needed)

## Custom DSL Expression Syntax (Phase 3)

Based on user preferences, the expression language will use:

**Syntax Elements:**
- Field access: `chunk.type` (dot notation)
- Array indexing: `items[-1]`, `items[0]` (brackets, Python-style negative indexing)
- Comparisons: `equals`, `not-equals`, `less-than`, `greater-than` (word operators)
- Logic: `AND`, `OR` (uppercase keywords)
- Byte arrays: Support all three formats
  - Array literal: `[73, 69, 78, 68]`
  - Hex literal: `0x49454E44`
  - String literal: `'IEND'`
- Special keywords: `eof` for end-of-file

**Example expressions:**
```yaml
# Check last chunk type
until: chunks[-1].type equals 'IEND'

# Multiple conditions
until: packet.flags equals 0x00 AND packet.length less-than 1500

# EOF check
until: eof

# Byte array comparison (all equivalent)
until: chunk.type equals [73, 69, 78, 68]
until: chunk.type equals 0x49454E44
until: chunk.type equals 'IEND'
```

## References
- Kaitai Struct: `repeat: expr`, `repeat: eos`, `repeat-expr`, `repeat-until`
- Protocol Buffers: `repeated` keyword
- Real-world formats: PNG spec, ZIP spec, ELF spec

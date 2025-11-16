# Phase 7.2/7.3: Padding, Alignment, and Bitfields

## Overview
Extend dezzy to support:
1. Fixed-size padding
2. Alignment-based padding
3. Bitfield types (sub-byte integers)

## 1. Fixed Padding

### Syntax
```yaml
fields:
  - name: header
    type: u32
  - name: _pad1
    padding: 4        # Skip exactly 4 bytes
  - name: data
    type: u64
```

### Semantics
- `padding: N` skips exactly N bytes
- Padding fields do NOT appear in generated structs
- During write, N zero bytes are written
- Backward compatible with existing `skip: field_name` for variable padding

### Implementation
- HIR: `HirField { skip: Some(Skip::Fixed(N)) }`
- LIR: `LirOperation::PadFixed { bytes: usize }`
- Read: `reader.skip(N)`
- Write: `writer.write_padding(N)` or `for i in 0..N { writer.write_le(0u8); }`

## 2. Alignment Padding

### Syntax
```yaml
fields:
  - name: magic
    type: u32
  - name: _align1
    align: 8          # Align position to 8-byte boundary
  - name: data
    type: u64
```

### Semantics
- `align: N` pads until `position % N == 0`
- If already aligned, no padding added
- Alignment fields do NOT appear in generated structs
- During write, calculate padding needed and write zeros

### Examples
```
Position 0: magic (4 bytes) -> position 4
Position 4: align 8 -> pad 4 bytes -> position 8
Position 8: data (8 bytes) -> position 16
```

### Implementation
- HIR: `HirField { skip: Some(Skip::Align(N)) }`
- LIR: `LirOperation::Align { boundary: usize }`
- Read:
  ```rust
  let padding = (boundary - (reader.position() % boundary)) % boundary;
  reader.skip(padding);
  ```
- Write:
  ```rust
  let padding = (boundary - (writer.position() % boundary)) % boundary;
  for _ in 0..padding { writer.write_le(0u8); }
  ```

## 3. Bitfields

### Syntax
```yaml
name: PacketHeader
bit_order: msb    # Optional, defaults to msb

types:
  - name: Flags
    type: struct
    fields:
      - name: version
        type: u3      # 3 bits
      - name: priority
        type: u2      # 2 bits
      - name: reserved
        type: u3      # 3 bits (total 1 byte)
      - name: length
        type: u16     # Next byte boundary, 2 bytes
```

### Semantics
- Types: `u1, u2, u3, u4, u5, u6, u7` and `i1, i2, i3, i4, i5, i6, i7`
- Bitfields are packed into bytes according to bit_order
- MSB-first (default): Most significant bit first (network byte order for bits)
- LSB-first: Least significant bit first (x86 style)
- Automatic byte boundary: Next non-bit field starts on new byte

### Bit Packing Examples

**MSB-first (default):**
```
Byte 0: [version:3][priority:2][reserved:3]
         76543210 bit positions
```

**LSB-first:**
```
Byte 0: [reserved:3][priority:2][version:3]
         76543210 bit positions
```

### Implementation
- HIR: Add `HirPrimitiveType::U1..U7, I1..I7`
- LIR: Add bit accumulator operations
  - `LirOperation::StartBitfield`
  - `LirOperation::ReadBits { dest, num_bits, signed }`
  - `LirOperation::WriteBits { src, num_bits }`
  - `LirOperation::EndBitfield`
- Code generation:
  - Track bit position within current byte
  - Accumulate bits during read/write
  - Flush to byte when crossing boundary

### C++ Implementation Sketch
```cpp
class BitReader {
    uint8_t current_byte = 0;
    int bits_remaining = 0;

    uint32_t read_bits(size_t num_bits) {
        uint32_t result = 0;
        while (num_bits > 0) {
            if (bits_remaining == 0) {
                current_byte = reader.read_le<uint8_t>();
                bits_remaining = 8;
            }
            int bits_to_read = std::min(num_bits, bits_remaining);
            // MSB-first extraction
            result = (result << bits_to_read) |
                     ((current_byte >> (bits_remaining - bits_to_read)) & ((1 << bits_to_read) - 1));
            bits_remaining -= bits_to_read;
            num_bits -= bits_to_read;
        }
        return result;
    }
};
```

## Combined Example

```yaml
name: NetworkPacket
endianness: big
bit_order: msb

types:
  - name: PacketHeader
    type: struct
    fields:
      - name: version
        type: u4
        doc: "Protocol version (4 bits)"
      - name: header_length
        type: u4
        doc: "Header length in 32-bit words (4 bits)"
      - name: type_of_service
        type: u8
        doc: "TOS field"
      - name: total_length
        type: u16
        doc: "Total packet length"
      - name: _align1
        align: 4
        doc: "Align to 4-byte boundary"
      - name: payload_size
        type: u32
      - name: payload
        type: blob(payload_size)
      - name: _pad1
        padding: 16
        doc: "Fixed 16-byte padding"
```

## Migration Path

### Existing Code (Phase 7.1)
```yaml
- name: _padding
  skip: padding_size  # Variable padding (KEEP)
  type: u8
```

### New Code (Phase 7.2/7.3)
```yaml
- name: _padding
  skip: padding_size  # Variable - no change

- name: _pad1
  padding: 4          # Fixed padding - NEW

- name: _align1
  align: 8            # Alignment - NEW

- name: flags
  type: u3            # Bitfield - NEW
```

## Implementation Plan

1. **Update schema**: Add `padding`, `align` to `YamlField`
2. **Update HIR**: Extend `Skip` enum with `Fixed(usize)` and `Align(usize)`
3. **Update parser**: Parse `padding` and `align` directives
4. **Add bitfield types**: Extend `HirPrimitiveType` with U1-U7, I1-I7
5. **Update pipeline**: Generate LIR operations for padding/alignment/bitfields
6. **Update backends**: Generate code for all three features
7. **Add tests**: Test cases for padding, alignment, bitfields, and combinations

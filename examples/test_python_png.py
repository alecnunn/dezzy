#!/usr/bin/env python3
"""Test the generated Python PNG chunk parser"""

import sys
sys.path.insert(0, 'examples/png_chunk_py')

from pngchunk import Chunk

def test_png_chunk():
    # Create a PNG chunk (IHDR-like)
    chunk = Chunk(
        length=5,
        chunk_type=[ord('I'), ord('H'), ord('D'), ord('R')],
        data=[0x00, 0x00, 0x01, 0x00, 0x08],
        crc=0x12345678
    )

    print("Original chunk:")
    print(f"  Length: {chunk.length}")
    print(f"  Type: {''.join(chr(c) for c in chunk.chunk_type)}")
    print(f"  Data: {chunk.data}")
    print(f"  CRC: 0x{chunk.crc:08x}")

    # Serialize
    serialized = chunk.write()
    print(f"\nSerialized to {len(serialized)} bytes")
    print(f"  Hex: {serialized.hex(' ')}")

    # Expected: 4 (length) + 4 (type) + 5 (data) + 4 (crc) = 17 bytes
    assert len(serialized) == 17, f"Expected 17 bytes, got {len(serialized)}"

    # Deserialize
    parsed, bytes_read = Chunk.read(serialized)
    print(f"\nParsed chunk ({bytes_read} bytes read):")
    print(f"  Length: {parsed.length}")
    print(f"  Type: {''.join(chr(c) for c in parsed.chunk_type)}")
    print(f"  Data: {parsed.data}")
    print(f"  CRC: 0x{parsed.crc:08x}")

    # Verify
    assert parsed.length == chunk.length, "Length mismatch"
    assert parsed.chunk_type == chunk.chunk_type, "Chunk type mismatch"
    assert parsed.data == chunk.data, "Data mismatch"
    assert parsed.crc == chunk.crc, "CRC mismatch"

    print("\nALL TESTS PASSED! Python code generator works correctly.")

if __name__ == "__main__":
    test_png_chunk()

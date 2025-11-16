#!/usr/bin/env python3
"""Test the container format with array of structs"""

import sys
sys.path.insert(0, 'examples/container_py')

from container import Chunk, Container

def test_container():
    # Create a container with 3 chunks
    chunks = [
        Chunk(
            length=5,
            chunk_type=[ord('I'), ord('H'), ord('D'), ord('R')],
            data=[0x00, 0x00, 0x01, 0x00, 0x08],
            crc=0x12345678
        ),
        Chunk(
            length=3,
            chunk_type=[ord('D'), ord('A'), ord('T'), ord('A')],
            data=[0xAA, 0xBB, 0xCC],
            crc=0x87654321
        ),
        Chunk(
            length=0,
            chunk_type=[ord('I'), ord('E'), ord('N'), ord('D')],
            data=[],
            crc=0xFFFFFFFF
        ),
    ]

    container = Container(
        num_chunks=3,
        chunks=chunks
    )

    print("Original container:")
    print(f"  Num chunks: {container.num_chunks}")
    for i, chunk in enumerate(container.chunks):
        print(f"  Chunk {i}:")
        print(f"    Type: {''.join(chr(c) for c in chunk.chunk_type)}")
        print(f"    Length: {chunk.length}")
        print(f"    CRC: 0x{chunk.crc:08x}")

    # Serialize
    serialized = container.write()
    print(f"\nSerialized to {len(serialized)} bytes")

    # Expected size:
    # 4 (num_chunks) + 3 * (4 + 4 + data_len + 4)
    # = 4 + 3 * 12 + (5 + 3 + 0) = 4 + 36 + 8 = 48 bytes
    expected_size = 4 + (4 + 4 + 5 + 4) + (4 + 4 + 3 + 4) + (4 + 4 + 0 + 4)
    assert len(serialized) == expected_size, f"Expected {expected_size} bytes, got {len(serialized)}"

    # Deserialize
    parsed, bytes_read = Container.read(serialized)
    print(f"\nParsed container ({bytes_read} bytes read):")
    print(f"  Num chunks: {parsed.num_chunks}")

    # Verify
    assert parsed.num_chunks == container.num_chunks, "Num chunks mismatch"
    assert len(parsed.chunks) == len(container.chunks), "Chunk count mismatch"

    for i in range(len(container.chunks)):
        orig = container.chunks[i]
        pars = parsed.chunks[i]
        print(f"  Chunk {i}:")
        print(f"    Type: {''.join(chr(c) for c in pars.chunk_type)}")
        print(f"    Length: {pars.length}")
        print(f"    CRC: 0x{pars.crc:08x}")

        assert pars.length == orig.length, f"Chunk {i} length mismatch"
        assert pars.chunk_type == orig.chunk_type, f"Chunk {i} type mismatch"
        assert pars.data == orig.data, f"Chunk {i} data mismatch"
        assert pars.crc == orig.crc, f"Chunk {i} CRC mismatch"

    print("\nALL TESTS PASSED! Struct arrays work correctly.")

if __name__ == "__main__":
    test_container()

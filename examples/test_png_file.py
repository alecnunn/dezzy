#!/usr/bin/env python3
"""Test parsing a complete PNG file with multiple chunks"""

import sys
sys.path.insert(0, 'examples/png_file_py')

from pngfile import PNGFile, Chunk

def test_create_png():
    """Test creating and serializing a minimal PNG file"""
    # PNG signature
    signature = [137, 80, 78, 71, 13, 10, 26, 10]

    # Create chunks
    chunks = [
        Chunk(
            length=13,
            chunk_type=[ord('I'), ord('H'), ord('D'), ord('R')],
            data=[0] * 13,  # Minimal IHDR data
            crc=0x00000000
        ),
        Chunk(
            length=0,
            chunk_type=[ord('I'), ord('E'), ord('N'), ord('D')],
            data=[],
            crc=0xAE426082  # Standard IEND CRC
        ),
    ]

    png = PNGFile(
        signature=signature,
        num_chunks=2,
        chunks=chunks
    )

    print("Original PNG:")
    print(f"  Signature: {png.signature}")
    print(f"  Num chunks: {png.num_chunks}")
    for i, chunk in enumerate(png.chunks):
        chunk_type = ''.join(chr(c) for c in chunk.chunk_type)
        print(f"  Chunk {i}: {chunk_type} (length={chunk.length})")

    # Serialize
    serialized = png.write()
    print(f"\nSerialized to {len(serialized)} bytes")

    # Expected size:
    # 8 (signature) + 4 (num_chunks) + (4+4+13+4) + (4+4+0+4) = 8 + 4 + 25 + 12 = 49 bytes
    expected = 8 + 4 + (4 + 4 + 13 + 4) + (4 + 4 + 0 + 4)
    assert len(serialized) == expected, f"Expected {expected} bytes, got {len(serialized)}"

    # Deserialize
    parsed, bytes_read = PNGFile.read(serialized)
    print(f"\nParsed PNG ({bytes_read} bytes read):")
    print(f"  Signature: {parsed.signature}")
    print(f"  Num chunks: {parsed.num_chunks}")

    # Verify
    assert parsed.signature == png.signature, "Signature mismatch"
    assert parsed.num_chunks == png.num_chunks, "Num chunks mismatch"
    assert len(parsed.chunks) == len(png.chunks), "Chunk count mismatch"

    for i in range(len(png.chunks)):
        orig = png.chunks[i]
        pars = parsed.chunks[i]
        chunk_type = ''.join(chr(c) for c in pars.chunk_type)
        print(f"  Chunk {i}: {chunk_type} (length={pars.length})")

        assert pars.length == orig.length, f"Chunk {i} length mismatch"
        assert pars.chunk_type == orig.chunk_type, f"Chunk {i} type mismatch"
        assert pars.data == orig.data, f"Chunk {i} data mismatch"
        assert pars.crc == orig.crc, f"Chunk {i} CRC mismatch"

    print("\nALL TESTS PASSED! PNG file format works correctly.")

if __name__ == "__main__":
    test_create_png()

#!/usr/bin/env python3
"""Test the generated PNG parser with until-condition arrays"""

import sys
sys.path.insert(0, '.')

from png import PNG, Chunk

def test_png_roundtrip():
    """Test creating a minimal PNG and parsing it back"""
    print("Testing PNG roundtrip...")

    # PNG signature
    signature = [137, 80, 78, 71, 13, 10, 26, 10]

    # Create IHDR chunk (Image Header)
    ihdr_data = [0, 0, 0, 1,  # Width: 1
                 0, 0, 0, 1,  # Height: 1
                 8,            # Bit depth
                 2,            # Color type (RGB)
                 0,            # Compression
                 0,            # Filter
                 0]            # Interlace
    ihdr = Chunk(
        length=len(ihdr_data),
        chunk_type=[73, 72, 68, 82],  # 'IHDR'
        data=ihdr_data,
        crc=0  # Simplified - real PNG would calculate CRC
    )

    # Create IEND chunk (End marker)
    iend = Chunk(
        length=0,
        chunk_type=[73, 69, 78, 68],  # 'IEND'
        data=[],
        crc=0
    )

    # Create PNG
    png = PNG(signature=signature, chunks=[ihdr, iend])

    # Write to bytes
    png_bytes = png.write()
    print(f"Generated PNG: {len(png_bytes)} bytes")

    # Read it back
    parsed_png, bytes_read = PNG.read(png_bytes)
    print(f"Parsed PNG: {bytes_read} bytes read")

    # Verify
    assert parsed_png.signature == signature, "Signature mismatch"
    assert len(parsed_png.chunks) == 2, f"Expected 2 chunks, got {len(parsed_png.chunks)}"

    # Verify IHDR
    assert parsed_png.chunks[0].chunk_type == [73, 72, 68, 82], "First chunk should be IHDR"
    assert parsed_png.chunks[0].length == len(ihdr_data), "IHDR length mismatch"

    # Verify IEND (this is the critical test - until condition should stop here)
    assert parsed_png.chunks[1].chunk_type == [73, 69, 78, 68], "Last chunk should be IEND"
    assert parsed_png.chunks[1].length == 0, "IEND should have no data"

    print("[OK] PNG signature correct")
    print("[OK] Chunk count correct (until-condition stopped at IEND)")
    print("[OK] IHDR chunk parsed correctly")
    print("[OK] IEND chunk parsed correctly")
    print("\nAll tests passed!")

if __name__ == "__main__":
    test_png_roundtrip()

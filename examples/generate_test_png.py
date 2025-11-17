#!/usr/bin/env python3
"""Generate a minimal valid PNG file for testing."""

import struct
import zlib

def crc32(data):
    """Calculate CRC32 for PNG chunk."""
    return zlib.crc32(data) & 0xFFFFFFFF

def write_chunk(f, chunk_type, data):
    """Write a PNG chunk with proper CRC."""
    length = len(data)
    f.write(struct.pack('>I', length))
    f.write(chunk_type)
    f.write(data)
    crc = crc32(chunk_type + data)
    f.write(struct.pack('>I', crc))

def generate_minimal_png(filename):
    """Generate a minimal 1x1 red pixel PNG."""
    with open(filename, 'wb') as f:
        # PNG signature
        f.write(bytes([137, 80, 78, 71, 13, 10, 26, 10]))

        # IHDR chunk (13 bytes)
        ihdr_data = struct.pack('>IIBBBBB',
            1,      # Width
            1,      # Height
            8,      # Bit depth
            2,      # Color type (2 = RGB)
            0,      # Compression method
            0,      # Filter method
            0       # Interlace method
        )
        write_chunk(f, b'IHDR', ihdr_data)

        # IDAT chunk (image data: 1 red pixel)
        # Filter byte (0 = none) + RGB data (255, 0, 0)
        raw_data = bytes([0, 255, 0, 0])
        compressed = zlib.compress(raw_data)
        write_chunk(f, b'IDAT', compressed)

        # IEND chunk (no data)
        write_chunk(f, b'IEND', b'')

    print(f"Generated {filename}")

if __name__ == '__main__':
    generate_minimal_png('examples/test.png')

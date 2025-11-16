#!/usr/bin/env python3
"""Test runner for assertion validation in test_assert.py"""

import struct
import sys
from test_assert import Header


def test_valid_header():
    """Test parsing a valid header with all assertions satisfied"""
    print("Test: Valid header... ", end="")

    # Create valid header data: magic=[0x89, 0x50, 0x4E, 0x47], version=1, width=100, height=200, flags=3
    data = bytes([
        0x89, 0x50, 0x4E, 0x47,  # magic (PNG signature)
        0x00, 0x01,              # version = 1 (big endian)
        0x00, 0x00, 0x00, 0x64,  # width = 100 (big endian)
        0x00, 0x00, 0x00, 0xC8,  # height = 200 (big endian)
        0x03                      # flags = 3 (valid range 0-7)
    ])

    header, bytes_read = Header.read(data)

    assert header.magic == [0x89, 0x50, 0x4E, 0x47]
    assert header.version == 1
    assert header.width == 100
    assert header.height == 200
    assert header.flags == 3
    assert bytes_read == 15

    print("PASSED")


def test_invalid_magic():
    """Test that invalid magic number triggers assertion"""
    print("Test: Invalid magic number... ", end="")

    # Invalid magic number
    data = bytes([
        0x00, 0x00, 0x00, 0x00,  # wrong magic
        0x00, 0x01,
        0x00, 0x00, 0x00, 0x64,
        0x00, 0x00, 0x00, 0xC8,
        0x03
    ])

    try:
        Header.read(data)
        print("FAILED (should have thrown)")
        sys.exit(1)
    except ValueError as e:
        msg = str(e)
        assert "magic" in msg
        print(f"PASSED (caught: {e})")


def test_invalid_version():
    """Test that version < 1 triggers assertion"""
    print("Test: Invalid version (must be >= 1)... ", end="")

    # version = 0 (invalid)
    data = bytes([
        0x89, 0x50, 0x4E, 0x47,
        0x00, 0x00,              # version = 0 (invalid)
        0x00, 0x00, 0x00, 0x64,
        0x00, 0x00, 0x00, 0xC8,
        0x03
    ])

    try:
        Header.read(data)
        print("FAILED (should have thrown)")
        sys.exit(1)
    except ValueError as e:
        msg = str(e)
        assert "version" in msg
        print(f"PASSED (caught: {e})")


def test_invalid_width():
    """Test that width = 0 triggers assertion"""
    print("Test: Invalid width (must be > 0)... ", end="")

    # width = 0 (invalid)
    data = bytes([
        0x89, 0x50, 0x4E, 0x47,
        0x00, 0x01,
        0x00, 0x00, 0x00, 0x00,  # width = 0 (invalid)
        0x00, 0x00, 0x00, 0xC8,
        0x03
    ])

    try:
        Header.read(data)
        print("FAILED (should have thrown)")
        sys.exit(1)
    except ValueError as e:
        msg = str(e)
        assert "width" in msg
        print(f"PASSED (caught: {e})")


def test_invalid_height():
    """Test that height = 0 triggers assertion"""
    print("Test: Invalid height (must be > 0)... ", end="")

    # height = 0 (invalid)
    data = bytes([
        0x89, 0x50, 0x4E, 0x47,
        0x00, 0x01,
        0x00, 0x00, 0x00, 0x64,
        0x00, 0x00, 0x00, 0x00,  # height = 0 (invalid)
        0x03
    ])

    try:
        Header.read(data)
        print("FAILED (should have thrown)")
        sys.exit(1)
    except ValueError as e:
        msg = str(e)
        assert "height" in msg
        print(f"PASSED (caught: {e})")


def test_invalid_flags_too_high():
    """Test that flags > 7 triggers assertion"""
    print("Test: Invalid flags (must be in range 0-7)... ", end="")

    # flags = 8 (out of range)
    data = bytes([
        0x89, 0x50, 0x4E, 0x47,
        0x00, 0x01,
        0x00, 0x00, 0x00, 0x64,
        0x00, 0x00, 0x00, 0xC8,
        0x08                      # flags = 8 (invalid, must be 0-7)
    ])

    try:
        Header.read(data)
        print("FAILED (should have thrown)")
        sys.exit(1)
    except ValueError as e:
        msg = str(e)
        assert "flags" in msg
        print(f"PASSED (caught: {e})")


def test_roundtrip():
    """Test write/read roundtrip preserves data"""
    print("Test: Write/Read roundtrip... ", end="")

    # Create a header
    original = Header(
        magic=[0x89, 0x50, 0x4E, 0x47],
        version=2,
        width=1920,
        height=1080,
        flags=5
    )

    # Write it
    data = original.write()

    # Read it back
    read_back, bytes_read = Header.read(data)

    # Verify
    assert read_back.magic == original.magic
    assert read_back.version == original.version
    assert read_back.width == original.width
    assert read_back.height == original.height
    assert read_back.flags == original.flags

    print("PASSED")


if __name__ == "__main__":
    print("=== Testing Assertion Validation ===\n")

    test_valid_header()
    test_invalid_magic()
    test_invalid_version()
    test_invalid_width()
    test_invalid_height()
    test_invalid_flags_too_high()
    test_roundtrip()

    print("\n=== All tests passed! ===")

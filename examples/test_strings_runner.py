#!/usr/bin/env python3
"""Test runner for string types"""

from test_strings import FileHeader

def main():
    print("=== Testing String Types ===\n")

    # Create test data
    original = FileHeader(
        signature="DEZZ",
        name_len=8,
        filename="test.dat",
        path="/usr/local/bin"
    )

    # Write to bytes
    data = original.write()

    print(f"Wrote {len(data)} bytes")
    print(f"  signature: \"{original.signature}\"")
    print(f"  name_len: {original.name_len}")
    print(f"  filename: \"{original.filename}\"")
    print(f"  path: \"{original.path}\"\n")

    # Read it back
    parsed, bytes_read = FileHeader.read(data)

    print("Read back:")
    print(f"  signature: \"{parsed.signature}\"")
    print(f"  name_len: {parsed.name_len}")
    print(f"  filename: \"{parsed.filename}\"")
    print(f"  path: \"{parsed.path}\"\n")

    # Verify
    assert parsed.signature == original.signature, "Signature mismatch"
    assert parsed.name_len == original.name_len, "Name length mismatch"
    assert parsed.filename == original.filename, "Filename mismatch"
    assert parsed.path == original.path, "Path mismatch"

    print("=== All string tests passed! ===")

if __name__ == "__main__":
    main()

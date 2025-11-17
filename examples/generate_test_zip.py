#!/usr/bin/env python3
"""Generate a minimal ZIP file for testing."""

import zipfile
import os

def generate_test_zip(filename):
    """Create a simple ZIP with a few text files."""
    with zipfile.ZipFile(filename, 'w', zipfile.ZIP_DEFLATED) as zf:
        # Add a text file
        zf.writestr('hello.txt', 'Hello, World!')

        # Add another file
        zf.writestr('data.txt', 'This is test data.\n' * 10)

        # Add a file with a comment
        info = zipfile.ZipInfo('readme.txt')
        info.comment = b'This is a readme file'
        zf.writestr(info, 'README: This is a test ZIP file.')

    print(f"Generated {filename}")
    print(f"Size: {os.path.getsize(filename)} bytes")

if __name__ == '__main__':
    generate_test_zip('examples/test.zip')

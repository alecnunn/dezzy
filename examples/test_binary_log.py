#!/usr/bin/env python3
"""Test the binary log format with until-EOF arrays"""

import sys
import time
sys.path.insert(0, 'examples/log_py')

from binarylog import LogEntry, LogFile

def test_log_file():
    # Create log entries
    entries = [
        LogEntry(
            timestamp=1700000000000000,  # 2023-11-14
            level=1,  # INFO
            message_length=11,
            message=[ord(c) for c in "Hello World"]
        ),
        LogEntry(
            timestamp=1700000001000000,
            level=2,  # WARN
            message_length=13,
            message=[ord(c) for c in "Warning: test"]
        ),
        LogEntry(
            timestamp=1700000002000000,
            level=3,  # ERROR
            message_length=17,
            message=[ord(c) for c in "Error occurred!!!"]
        ),
    ]

    log_file = LogFile(entries=entries)

    print("Original log file:")
    print(f"  {len(log_file.entries)} entries")
    for i, entry in enumerate(log_file.entries):
        level_names = ["DEBUG", "INFO", "WARN", "ERROR"]
        message = ''.join(chr(c) for c in entry.message)
        print(f"  Entry {i}: [{level_names[entry.level]}] {message}")

    # Serialize
    serialized = log_file.write()
    print(f"\nSerialized to {len(serialized)} bytes")

    # Expected size:
    # Entry 0: 8 (timestamp) + 1 (level) + 2 (message_length) + 11 (message) = 22 bytes
    # Entry 1: 8 + 1 + 2 + 13 = 24 bytes
    # Entry 2: 8 + 1 + 2 + 17 = 28 bytes
    # Total: 22 + 24 + 28 = 74 bytes
    expected = 22 + 24 + 28
    assert len(serialized) == expected, f"Expected {expected} bytes, got {len(serialized)}"

    # Deserialize
    parsed, bytes_read = LogFile.read(serialized)
    print(f"\nParsed log file ({bytes_read} bytes read):")
    print(f"  {len(parsed.entries)} entries")

    # Verify
    assert len(parsed.entries) == len(log_file.entries), "Entry count mismatch"

    for i in range(len(log_file.entries)):
        orig = log_file.entries[i]
        pars = parsed.entries[i]
        level_names = ["DEBUG", "INFO", "WARN", "ERROR"]
        message = ''.join(chr(c) for c in pars.message)
        print(f"  Entry {i}: [{level_names[pars.level]}] {message}")

        assert pars.timestamp == orig.timestamp, f"Entry {i} timestamp mismatch"
        assert pars.level == orig.level, f"Entry {i} level mismatch"
        assert pars.message_length == orig.message_length, f"Entry {i} message_length mismatch"
        assert pars.message == orig.message, f"Entry {i} message mismatch"

    print("\nALL TESTS PASSED! Until-EOF arrays work correctly.")

if __name__ == "__main__":
    test_log_file()

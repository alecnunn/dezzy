#!/usr/bin/env python3
import sys
sys.path.insert(0, 'examples')
import test_enum

# Test all enum values
tests = [
    (test_enum.Status.OK, 100),
    (test_enum.Status.ERROR, 200),
    (test_enum.Status.PENDING, 300),
]

all_pass = True
for status, val in tests:
    msg = test_enum.Message(status, val)
    data = msg.write()
    msg2, bytes_read = test_enum.Message.read(data)

    if msg2.status != status or msg2.value != val or bytes_read != 5:
        all_pass = False
        print(f'Test {status.name}={status.value}, val={val}: FAIL')
        print(f'  Expected: status={status}, value={val}, bytes_read=5')
        print(f'  Got: status={msg2.status}, value={msg2.value}, bytes_read={bytes_read}')
    else:
        print(f'Test {status.name}={status.value}, val={val}: PASS')

if all_pass:
    print('\nAll tests passed!')
else:
    print('\nSome tests failed!')
    sys.exit(1)

#include "test_bitfields.h"
#include <iostream>
#include <vector>
#include <iomanip>

int main() {
    using namespace bitfieldtest;

    // Create test flags
    Flags flags;
    flags.version = 5;  // 3 bits, max value 7
    flags.compressed = 1;  // 1 bit
    flags.encrypted = 0;  // 1 bit
    flags.reserved = 3;  // 3 bits
    flags.value = 0xDEADBEEF;

    std::cout << "Original values:" << std::endl;
    std::cout << "  version: " << (int)flags.version << std::endl;
    std::cout << "  compressed: " << (int)flags.compressed << std::endl;
    std::cout << "  encrypted: " << (int)flags.encrypted << std::endl;
    std::cout << "  reserved: " << (int)flags.reserved << std::endl;
    std::cout << "  value: 0x" << std::hex << flags.value << std::dec << std::endl;

    // Write to bytes
    Writer writer;
    flags.write(writer);
    std::vector<uint8_t> data = writer.finish();

    std::cout << "\nWrote " << data.size() << " bytes: ";
    for (uint8_t byte : data) {
        std::cout << std::hex << std::setw(2) << std::setfill('0') << (int)byte << " ";
    }
    std::cout << std::dec << std::endl;

    // Read back
    try {
        Reader reader(data);
        Flags read_flags = Flags::read(reader);

        std::cout << "\nRead values:" << std::endl;
        std::cout << "  version: " << (int)read_flags.version << std::endl;
        std::cout << "  compressed: " << (int)read_flags.compressed << std::endl;
        std::cout << "  encrypted: " << (int)read_flags.encrypted << std::endl;
        std::cout << "  reserved: " << (int)read_flags.reserved << std::endl;
        std::cout << "  value: 0x" << std::hex << read_flags.value << std::dec << std::endl;

        // Verify
        bool success = true;
        if (read_flags.version != flags.version) {
            std::cerr << "ERROR: version mismatch!" << std::endl;
            success = false;
        }
        if (read_flags.compressed != flags.compressed) {
            std::cerr << "ERROR: compressed mismatch!" << std::endl;
            success = false;
        }
        if (read_flags.encrypted != flags.encrypted) {
            std::cerr << "ERROR: encrypted mismatch!" << std::endl;
            success = false;
        }
        if (read_flags.reserved != flags.reserved) {
            std::cerr << "ERROR: reserved mismatch!" << std::endl;
            success = false;
        }
        if (read_flags.value != flags.value) {
            std::cerr << "ERROR: value mismatch!" << std::endl;
            success = false;
        }

        if (success) {
            std::cout << "\n✓ Test PASSED!" << std::endl;
            return 0;
        } else {
            std::cout << "\n✗ Test FAILED!" << std::endl;
            return 1;
        }
    } catch (const ParseError& e) {
        std::cerr << "Parse error: " << e.what() << std::endl;
        return 1;
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }
}

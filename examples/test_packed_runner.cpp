#include "test_packed_format.h"
#include <iostream>
#include <vector>

int main() {
    using namespace packedformat;

    // Create a test header
    PackedHeader header;
    header.magic = 0x50414B44;  // "PAKD"
    header.version = 5;  // 3 bits, max value 7
    header.compressed = 1;  // 1 bit
    header.encrypted = 0;  // 1 bit
    header.reserved_bits = 0;  // 3 bits
    header.data_size = 1024;
    header.data_offset = 0x1000;
    header.priority = 2;  // 2 bits, max value 3
    header.status = -2;  // 3-bit signed, range -4 to 3
    header.flags = 7;  // 3 bits, max value 7
    header.checksum = 0xDEADBEEF;

    // Write header to bytes
    Writer writer;
    header.write(writer);
    std::vector<uint8_t> data = writer.finish();

    std::cout << "Wrote " << data.size() << " bytes" << std::endl;

    // Read header back
    try {
        Reader reader(data);
        PackedHeader read_header = PackedHeader::read(reader);

        // Verify
        std::cout << "Magic: 0x" << std::hex << read_header.magic << std::endl;
    std::cout << "Version: " << std::dec << (int)read_header.version << std::endl;
    std::cout << "Compressed: " << (int)read_header.compressed << std::endl;
    std::cout << "Encrypted: " << (int)read_header.encrypted << std::endl;
    std::cout << "Data size: " << read_header.data_size << std::endl;
    std::cout << "Data offset: 0x" << std::hex << read_header.data_offset << std::endl;
    std::cout << "Priority: " << std::dec << (int)read_header.priority << std::endl;
    std::cout << "Status: " << (int)read_header.status << std::endl;
        std::cout << "Flags: " << (int)read_header.flags << std::endl;
        std::cout << "Checksum: 0x" << std::hex << read_header.checksum << std::endl;

        std::cout << "\nTest passed!" << std::endl;
    } catch (const ParseError& e) {
        std::cerr << "Parse error: " << e.what() << std::endl;
        return 1;
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }

    return 0;
}

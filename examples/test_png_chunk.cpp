#include "png_chunk.hpp/pngchunk.hpp"
#include <iostream>
#include <iomanip>
#include <vector>
#include <cstdint>

void print_bytes(const std::vector<uint8_t>& data) {
    for (size_t i = 0; i < data.size(); ++i) {
        std::cout << std::hex << std::setw(2) << std::setfill('0')
                  << static_cast<int>(data[i]) << " ";
        if ((i + 1) % 16 == 0) std::cout << "\n";
    }
    if (data.size() % 16 != 0) std::cout << "\n";
}

int main() {
    using namespace pngchunk;

    // Create a PNG chunk similar to an IHDR chunk (but simplified)
    // IHDR typically has 13 bytes of data
    Chunk chunk;
    chunk.length = 5;  // 5 bytes of data
    chunk.chunk_type = {'I', 'H', 'D', 'R'};
    chunk.data = {0x00, 0x00, 0x01, 0x00, 0x08};  // Some fake image metadata
    chunk.crc = 0x12345678;  // Fake CRC for testing

    std::cout << "Original chunk:\n";
    std::cout << "  Length: " << chunk.length << "\n";
    std::cout << "  Type: " << chunk.chunk_type[0] << chunk.chunk_type[1]
              << chunk.chunk_type[2] << chunk.chunk_type[3] << "\n";
    std::cout << "  Data size: " << chunk.data.size() << "\n";
    std::cout << "  CRC: 0x" << std::hex << chunk.crc << std::dec << "\n\n";

    // Serialize to bytes
    Writer writer;
    chunk.write(writer);
    std::vector<uint8_t> serialized = writer.finish();

    std::cout << "Serialized bytes (" << serialized.size() << " bytes):\n";
    print_bytes(serialized);
    std::cout << "\n";

    // Expected format:
    // 4 bytes: length (big-endian)
    // 4 bytes: chunk_type
    // N bytes: data (where N = length)
    // 4 bytes: crc (big-endian)
    // Total: 4 + 4 + 5 + 4 = 17 bytes

    if (serialized.size() != 17) {
        std::cerr << "ERROR: Expected 17 bytes, got " << serialized.size() << "\n";
        return 1;
    }

    // Deserialize back
    Reader reader(serialized);
    Chunk parsed = Chunk::read(reader);

    std::cout << "Parsed chunk:\n";
    std::cout << "  Length: " << parsed.length << "\n";
    std::cout << "  Type: " << parsed.chunk_type[0] << parsed.chunk_type[1]
              << parsed.chunk_type[2] << parsed.chunk_type[3] << "\n";
    std::cout << "  Data size: " << parsed.data.size() << "\n";
    std::cout << "  CRC: 0x" << std::hex << parsed.crc << std::dec << "\n\n";

    // Verify all fields match
    if (parsed.length != chunk.length) {
        std::cerr << "ERROR: Length mismatch\n";
        return 1;
    }
    if (parsed.chunk_type != chunk.chunk_type) {
        std::cerr << "ERROR: Chunk type mismatch\n";
        return 1;
    }
    if (parsed.data != chunk.data) {
        std::cerr << "ERROR: Data mismatch\n";
        return 1;
    }
    if (parsed.crc != chunk.crc) {
        std::cerr << "ERROR: CRC mismatch\n";
        return 1;
    }

    std::cout << "✓ All tests passed! Variable-length arrays work correctly.\n";
    return 0;
}

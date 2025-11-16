#include "container_cpp/container.hpp"
#include <iostream>
#include <iomanip>
#include <cassert>

using namespace container;

void print_chunk(const Chunk& chunk, int index) {
    std::cout << "  Chunk " << index << ":\n";
    std::cout << "    Type: ";
    for (auto c : chunk.chunk_type) {
        std::cout << static_cast<char>(c);
    }
    std::cout << "\n";
    std::cout << "    Length: " << chunk.length << "\n";
    std::cout << "    CRC: 0x" << std::hex << std::setw(8) << std::setfill('0')
              << chunk.crc << std::dec << "\n";
}

int main() {
    // Create a container with 3 chunks
    Container original;
    original.num_chunks = 3;
    original.chunks.resize(3);

    // Chunk 0: IHDR
    original.chunks[0].length = 5;
    original.chunks[0].chunk_type = {'I', 'H', 'D', 'R'};
    original.chunks[0].data = {0x00, 0x00, 0x01, 0x00, 0x08};
    original.chunks[0].crc = 0x12345678;

    // Chunk 1: DATA
    original.chunks[1].length = 3;
    original.chunks[1].chunk_type = {'D', 'A', 'T', 'A'};
    original.chunks[1].data = {0xAA, 0xBB, 0xCC};
    original.chunks[1].crc = 0x87654321;

    // Chunk 2: IEND
    original.chunks[2].length = 0;
    original.chunks[2].chunk_type = {'I', 'E', 'N', 'D'};
    original.chunks[2].data = {};
    original.chunks[2].crc = 0xFFFFFFFF;

    std::cout << "Original container:\n";
    std::cout << "  Num chunks: " << original.num_chunks << "\n";
    for (size_t i = 0; i < original.chunks.size(); ++i) {
        print_chunk(original.chunks[i], i);
    }

    // Serialize
    Writer writer;
    original.write(writer);
    auto serialized = writer.finish();

    std::cout << "\nSerialized to " << serialized.size() << " bytes\n";

    // Expected: 4 + (4+4+5+4) + (4+4+3+4) + (4+4+0+4) = 48 bytes
    assert(serialized.size() == 48);

    // Deserialize
    Reader reader(serialized);
    auto parsed = Container::read(reader);

    std::cout << "\nParsed container:\n";
    std::cout << "  Num chunks: " << parsed.num_chunks << "\n";

    // Verify
    assert(parsed.num_chunks == original.num_chunks);
    assert(parsed.chunks.size() == original.chunks.size());

    for (size_t i = 0; i < original.chunks.size(); ++i) {
        print_chunk(parsed.chunks[i], i);

        assert(parsed.chunks[i].length == original.chunks[i].length);
        assert(parsed.chunks[i].chunk_type == original.chunks[i].chunk_type);
        assert(parsed.chunks[i].data == original.chunks[i].data);
        assert(parsed.chunks[i].crc == original.chunks[i].crc);
    }

    std::cout << "\nALL TESTS PASSED! C++ struct arrays work correctly.\n";
    return 0;
}

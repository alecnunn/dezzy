#include "png_simple.hpp/pngsimple.hpp"
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
    using namespace pngsimple;

    // Create a simplified PNG file
    PNGFile png;

    // PNG signature: 89 50 4E 47 0D 0A 1A 0A
    png.signature = {0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A};

    // IHDR chunk
    // Real IHDR has 13 bytes: width(4) height(4) depth(1) color(1) compression(1) filter(1) interlace(1)
    png.ihdr_chunk.length = 13;
    png.ihdr_chunk.chunk_type = {'I', 'H', 'D', 'R'};
    png.ihdr_chunk.data = {
        0x00, 0x00, 0x00, 0x20,  // Width: 32 pixels
        0x00, 0x00, 0x00, 0x20,  // Height: 32 pixels
        0x08,                     // Bit depth: 8
        0x02,                     // Color type: 2 (RGB)
        0x00,                     // Compression: 0 (deflate)
        0x00,                     // Filter: 0 (adaptive)
        0x00                      // Interlace: 0 (none)
    };
    png.ihdr_chunk.crc = 0x91bae829;  // Actual CRC for this IHDR

    std::cout << "Original PNG:\n";
    std::cout << "  Signature: ";
    for (auto b : png.signature) {
        std::cout << std::hex << std::setw(2) << std::setfill('0') << static_cast<int>(b) << " ";
    }
    std::cout << "\n";
    std::cout << "  IHDR length: " << std::dec << png.ihdr_chunk.length << "\n";
    std::cout << "  IHDR type: " << png.ihdr_chunk.chunk_type[0] << png.ihdr_chunk.chunk_type[1]
              << png.ihdr_chunk.chunk_type[2] << png.ihdr_chunk.chunk_type[3] << "\n";
    std::cout << "  IHDR data size: " << png.ihdr_chunk.data.size() << " bytes\n\n";

    // Serialize
    Writer writer;
    png.write(writer);
    std::vector<uint8_t> serialized = writer.finish();

    std::cout << "Serialized PNG (" << serialized.size() << " bytes):\n";
    print_bytes(serialized);
    std::cout << "\n";

    // Expected: 8 (signature) + 4 (length) + 4 (type) + 13 (data) + 4 (crc) = 33 bytes
    if (serialized.size() != 33) {
        std::cerr << "ERROR: Expected 33 bytes, got " << serialized.size() << "\n";
        return 1;
    }

    // Deserialize
    Reader reader(serialized);
    PNGFile parsed = PNGFile::read(reader);

    std::cout << "Parsed PNG:\n";
    std::cout << "  Signature: ";
    for (auto b : parsed.signature) {
        std::cout << std::hex << std::setw(2) << std::setfill('0') << static_cast<int>(b) << " ";
    }
    std::cout << "\n";
    std::cout << "  IHDR length: " << std::dec << parsed.ihdr_chunk.length << "\n";
    std::cout << "  IHDR type: " << parsed.ihdr_chunk.chunk_type[0] << parsed.ihdr_chunk.chunk_type[1]
              << parsed.ihdr_chunk.chunk_type[2] << parsed.ihdr_chunk.chunk_type[3] << "\n";
    std::cout << "  IHDR data size: " << parsed.ihdr_chunk.data.size() << " bytes\n\n";

    // Verify
    if (parsed.signature != png.signature) {
        std::cerr << "ERROR: Signature mismatch\n";
        return 1;
    }
    if (parsed.ihdr_chunk.length != png.ihdr_chunk.length) {
        std::cerr << "ERROR: IHDR length mismatch\n";
        return 1;
    }
    if (parsed.ihdr_chunk.chunk_type != png.ihdr_chunk.chunk_type) {
        std::cerr << "ERROR: IHDR type mismatch\n";
        return 1;
    }
    if (parsed.ihdr_chunk.data != png.ihdr_chunk.data) {
        std::cerr << "ERROR: IHDR data mismatch\n";
        return 1;
    }
    if (parsed.ihdr_chunk.crc != png.ihdr_chunk.crc) {
        std::cerr << "ERROR: IHDR CRC mismatch\n";
        return 1;
    }

    std::cout << "✓ PNG parsing successful! Nested structs with variable-length arrays work.\n";

    // Verify the signature matches real PNG signature
    std::vector<uint8_t> expected_sig = {0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A};
    bool sig_ok = true;
    for (size_t i = 0; i < 8; ++i) {
        if (parsed.signature[i] != expected_sig[i]) {
            sig_ok = false;
            break;
        }
    }

    if (sig_ok) {
        std::cout << "✓ PNG signature is valid!\n";
    } else {
        std::cerr << "ERROR: PNG signature is invalid\n";
        return 1;
    }

    return 0;
}

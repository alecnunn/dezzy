#include "png_simple.hpp/pngsimple.hpp"
#include <iostream>
#include <iomanip>
#include <fstream>
#include <vector>
#include <cstdint>

std::vector<uint8_t> read_file(const std::string& filename) {
    std::ifstream file(filename, std::ios::binary | std::ios::ate);
    if (!file) {
        throw std::runtime_error("Failed to open file: " + filename);
    }

    std::streamsize size = file.tellg();
    file.seekg(0, std::ios::beg);

    std::vector<uint8_t> buffer(size);
    if (!file.read(reinterpret_cast<char*>(buffer.data()), size)) {
        throw std::runtime_error("Failed to read file: " + filename);
    }

    return buffer;
}

void print_bytes(const uint8_t* data, size_t count) {
    for (size_t i = 0; i < count; ++i) {
        std::cout << std::hex << std::setw(2) << std::setfill('0')
                  << static_cast<int>(data[i]) << " ";
    }
    std::cout << std::dec;
}

void print_chunk_type(const std::array<uint8_t, 4>& type) {
    for (auto c : type) {
        if (c >= 32 && c < 127) {
            std::cout << static_cast<char>(c);
        } else {
            std::cout << "?";
        }
    }
}

int main() {
    using namespace pngsimple;

    try {
        // Read the real PNG file
        std::cout << "Reading icon.png...\n";
        std::vector<uint8_t> file_data = read_file("icon.png");
        std::cout << "File size: " << file_data.size() << " bytes\n\n";

        // Parse with our generated code
        Reader reader(file_data);
        PNGFile png = PNGFile::read(reader);

        // Validate PNG signature
        std::cout << "PNG Signature: ";
        print_bytes(png.signature.data(), 8);
        std::cout << "\n";

        // Expected PNG signature
        std::array<uint8_t, 8> expected_sig = {0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A};
        if (png.signature != expected_sig) {
            std::cerr << "ERROR: Invalid PNG signature!\n";
            return 1;
        }
        std::cout << "✓ PNG signature is valid\n\n";

        // Display IHDR chunk info
        std::cout << "IHDR Chunk:\n";
        std::cout << "  Length: " << png.ihdr_chunk.length << " bytes\n";
        std::cout << "  Type: ";
        print_chunk_type(png.ihdr_chunk.chunk_type);
        std::cout << "\n";

        // Validate chunk type
        std::array<uint8_t, 4> expected_ihdr = {'I', 'H', 'D', 'R'};
        if (png.ihdr_chunk.chunk_type != expected_ihdr) {
            std::cerr << "ERROR: Expected IHDR chunk!\n";
            return 1;
        }
        std::cout << "✓ IHDR chunk type is correct\n";

        // IHDR should be exactly 13 bytes
        if (png.ihdr_chunk.length != 13) {
            std::cerr << "ERROR: IHDR length should be 13, got " << png.ihdr_chunk.length << "\n";
            return 1;
        }
        std::cout << "✓ IHDR length is correct (13 bytes)\n";

        // Verify data size matches length
        if (png.ihdr_chunk.data.size() != png.ihdr_chunk.length) {
            std::cerr << "ERROR: Data size mismatch!\n";
            return 1;
        }
        std::cout << "✓ Data size matches length field\n\n";

        // Parse IHDR data (13 bytes)
        if (png.ihdr_chunk.data.size() >= 13) {
            uint32_t width = (png.ihdr_chunk.data[0] << 24) |
                            (png.ihdr_chunk.data[1] << 16) |
                            (png.ihdr_chunk.data[2] << 8) |
                            png.ihdr_chunk.data[3];

            uint32_t height = (png.ihdr_chunk.data[4] << 24) |
                             (png.ihdr_chunk.data[5] << 16) |
                             (png.ihdr_chunk.data[6] << 8) |
                             png.ihdr_chunk.data[7];

            uint8_t bit_depth = png.ihdr_chunk.data[8];
            uint8_t color_type = png.ihdr_chunk.data[9];
            uint8_t compression = png.ihdr_chunk.data[10];
            uint8_t filter = png.ihdr_chunk.data[11];
            uint8_t interlace = png.ihdr_chunk.data[12];

            std::cout << "IHDR Details:\n";
            std::cout << "  Width: " << width << " pixels\n";
            std::cout << "  Height: " << height << " pixels\n";
            std::cout << "  Bit depth: " << static_cast<int>(bit_depth) << "\n";
            std::cout << "  Color type: " << static_cast<int>(color_type);
            switch (color_type) {
                case 0: std::cout << " (Grayscale)"; break;
                case 2: std::cout << " (RGB)"; break;
                case 3: std::cout << " (Indexed)"; break;
                case 4: std::cout << " (Grayscale+Alpha)"; break;
                case 6: std::cout << " (RGBA)"; break;
                default: std::cout << " (Unknown)"; break;
            }
            std::cout << "\n";
            std::cout << "  Compression: " << static_cast<int>(compression) << "\n";
            std::cout << "  Filter: " << static_cast<int>(filter) << "\n";
            std::cout << "  Interlace: " << static_cast<int>(interlace) << "\n";
        }

        std::cout << "\n  CRC: 0x" << std::hex << png.ihdr_chunk.crc << std::dec << "\n\n";

        std::cout << "Bytes consumed: " << reader.position() << " of " << file_data.size() << "\n";
        std::cout << "Remaining: " << reader.remaining() << " bytes (additional chunks not parsed)\n\n";

        std::cout << "✓✓✓ Real PNG file parsed successfully! ✓✓✓\n";
        std::cout << "Variable-length arrays work correctly with real-world data.\n";

        return 0;

    } catch (const std::exception& e) {
        std::cerr << "ERROR: " << e.what() << "\n";
        return 1;
    }
}

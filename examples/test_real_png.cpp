#include "png.hpp"
#include <iostream>
#include <fstream>
#include <vector>
#include <iomanip>

using namespace png;

std::vector<uint8_t> read_file(const char* filename) {
    std::ifstream file(filename, std::ios::binary | std::ios::ate);
    if (!file) {
        throw std::runtime_error("Failed to open file");
    }

    std::streamsize size = file.tellg();
    file.seekg(0, std::ios::beg);

    std::vector<uint8_t> buffer(size);
    if (!file.read(reinterpret_cast<char*>(buffer.data()), size)) {
        throw std::runtime_error("Failed to read file");
    }

    return buffer;
}

void print_chunk_type(const std::array<uint8_t, 4>& type) {
    for (int i = 0; i < 4; i++) {
        if (type[i] >= 32 && type[i] < 127) {
            std::cout << (char)type[i];
        } else {
            std::cout << "?";
        }
    }
}

int main() {
    try {
        std::cout << "Reading real PNG file: examples/logo.png\n";

        auto data = read_file("examples/logo.png");
        std::cout << "File size: " << data.size() << " bytes\n\n";

        // Parse PNG
        Reader reader(data);
        PNG png = PNG::read(reader);

        std::cout << "Successfully parsed PNG!\n";
        std::cout << "Bytes read: " << reader.position() << " / " << data.size() << "\n\n";

        // Verify signature
        std::cout << "Signature: ";
        bool sig_valid = true;
        const uint8_t expected_sig[] = {137, 80, 78, 71, 13, 10, 26, 10};
        for (int i = 0; i < 8; i++) {
            std::cout << std::hex << std::setw(2) << std::setfill('0')
                     << (int)png.signature[i] << " ";
            if (png.signature[i] != expected_sig[i]) {
                sig_valid = false;
            }
        }
        std::cout << std::dec << (sig_valid ? "[OK]" : "[FAIL]") << "\n\n";

        // Display chunks
        std::cout << "Chunks found: " << png.chunks.size() << "\n";
        std::cout << "----------------------------------------\n";

        for (size_t i = 0; i < png.chunks.size(); i++) {
            const auto& chunk = png.chunks[i];
            std::cout << "#" << std::setw(2) << i << ": ";
            print_chunk_type(chunk.chunk_type);
            std::cout << " - " << chunk.length << " bytes";

            // Special handling for IHDR
            if (chunk.chunk_type[0] == 'I' && chunk.chunk_type[1] == 'H' &&
                chunk.chunk_type[2] == 'D' && chunk.chunk_type[3] == 'R' &&
                chunk.data.size() >= 13) {
                uint32_t width = (chunk.data[0] << 24) | (chunk.data[1] << 16) |
                                (chunk.data[2] << 8) | chunk.data[3];
                uint32_t height = (chunk.data[4] << 24) | (chunk.data[5] << 16) |
                                 (chunk.data[6] << 8) | chunk.data[7];
                std::cout << " [" << width << "x" << height << "]";
            }

            std::cout << "\n";
        }

        // Verify last chunk is IEND
        if (!png.chunks.empty()) {
            const auto& last = png.chunks.back();
            bool is_iend = (last.chunk_type[0] == 'I' && last.chunk_type[1] == 'E' &&
                           last.chunk_type[2] == 'N' && last.chunk_type[3] == 'D');
            std::cout << "\nLast chunk is IEND: " << (is_iend ? "[OK]" : "[FAIL]") << "\n";
        }

        std::cout << "\n✓ Real PNG parsed successfully!\n";
        return 0;

    } catch (const ParseError& e) {
        std::cerr << "Parse error: " << e.what() << "\n";
        return 1;
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << "\n";
        return 1;
    }
}

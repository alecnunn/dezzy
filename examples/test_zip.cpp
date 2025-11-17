#include "zip.hpp"
#include <iostream>
#include <fstream>
#include <vector>
#include <iomanip>

using namespace zip;

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

// Find End of Central Directory by searching for signature from end
size_t find_eocd(const std::vector<uint8_t>& data) {
    const uint32_t eocd_sig = 0x06054b50;

    // Search from end, going backwards
    // EOCD is at least 22 bytes, max comment is 65535 bytes
    size_t search_start = data.size() >= 65557 ? data.size() - 65557 : 0;

    for (size_t i = data.size() - 22; i >= search_start && i < data.size(); i--) {
        uint32_t sig = data[i] | (data[i+1] << 8) | (data[i+2] << 16) | (data[i+3] << 24);
        if (sig == eocd_sig) {
            return i;
        }
    }

    throw std::runtime_error("EOCD signature not found");
}

int main(int argc, char* argv[]) {
    try {
        const char* filename = argc > 1 ? argv[1] : "examples/test.zip";

        std::cout << "Reading ZIP file: " << filename << "\n";

        auto data = read_file(filename);
        std::cout << "File size: " << data.size() << " bytes\n\n";

        // Find and parse End of Central Directory
        size_t eocd_offset = find_eocd(data);
        std::cout << "Found EOCD at offset: " << eocd_offset << "\n";

        std::vector<uint8_t> eocd_data(data.begin() + eocd_offset, data.end());
        Reader reader(eocd_data);
        EndOfCentralDirectory eocd = EndOfCentralDirectory::read(reader);

        std::cout << "\nEnd of Central Directory:\n";
        std::cout << "  Disk number: " << eocd.disk_number << "\n";
        std::cout << "  Disk with CD: " << eocd.disk_with_cd << "\n";
        std::cout << "  Entries on this disk: " << eocd.num_entries_this_disk << "\n";
        std::cout << "  Total entries: " << eocd.num_entries_total << "\n";
        std::cout << "  Central directory size: " << eocd.cd_size << " bytes\n";
        std::cout << "  Central directory offset: " << eocd.cd_offset << "\n";
        std::cout << "  Comment length: " << eocd.comment_length << "\n";

        if (eocd.comment_length > 0) {
            std::cout << "  Comment: ";
            for (uint16_t i = 0; i < eocd.comment_length && i < eocd.comment.size(); i++) {
                char c = eocd.comment[i];
                std::cout << (c >= 32 && c < 127 ? c : '?');
            }
            std::cout << "\n";
        }

        // Try to parse a central directory entry
        if (eocd.cd_offset < data.size() && eocd.num_entries_total > 0) {
            std::cout << "\nFirst Central Directory Entry:\n";
            std::vector<uint8_t> cd_data(data.begin() + eocd.cd_offset, data.end());
            Reader cd_reader(cd_data);

            try {
                CentralDirectoryHeader cd_header = CentralDirectoryHeader::read(cd_reader);
                std::cout << "  Version made by: " << cd_header.version_made_by << "\n";
                std::cout << "  Version needed: " << cd_header.version_needed << "\n";
                std::cout << "  Compression: " << cd_header.compression_method << "\n";
                std::cout << "  Compressed size: " << cd_header.compressed_size << " bytes\n";
                std::cout << "  Uncompressed size: " << cd_header.uncompressed_size << " bytes\n";
                std::cout << "  Filename: ";
                for (uint16_t i = 0; i < cd_header.filename_length && i < cd_header.filename.size(); i++) {
                    char c = cd_header.filename[i];
                    std::cout << (c >= 32 && c < 127 ? c : '?');
                }
                std::cout << "\n";
                std::cout << "  Local header offset: " << cd_header.local_header_offset << "\n";
            } catch (const ParseError& e) {
                std::cout << "  (Could not parse: " << e.what() << ")\n";
            }
        }

        std::cout << "\n✓ ZIP structures parsed successfully!\n";
        return 0;

    } catch (const ParseError& e) {
        std::cerr << "Parse error: " << e.what() << "\n";
        return 1;
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << "\n";
        return 1;
    }
}

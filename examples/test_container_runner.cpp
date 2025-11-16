#include "test_container.hpp"
#include <iostream>
#include <cassert>
#include <cstring>

int main() {
    using namespace testcontainer;

    std::cout << "=== Container Format Test ===" << std::endl;

    // Create a container with some file entries
    Container container;
    container.magic = 0x434E5452;  // "CNTR"
    container.num_entries = 3;

    // Entry 1: text file
    FileEntry entry1;
    entry1.filename_len = 8;
    entry1.filename = "test.txt";
    entry1.file_size = 13;
    entry1.file_data = {72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33}; // "Hello, World!"
    entry1.padding_size = 3;  // 3 bytes of padding to skip

    // Entry 2: binary data
    FileEntry entry2;
    entry2.filename_len = 8;
    entry2.filename = "data.bin";
    entry2.file_size = 5;
    entry2.file_data = {0xDE, 0xAD, 0xBE, 0xEF, 0x00};
    entry2.padding_size = 0;  // No padding

    // Entry 3: empty file
    FileEntry entry3;
    entry3.filename_len = 9;
    entry3.filename = "empty.txt";
    entry3.file_size = 0;
    entry3.file_data = {};
    entry3.padding_size = 16;  // 16 bytes of padding

    container.entries = {entry1, entry2, entry3};

    std::cout << "Created container with " << container.num_entries << " entries" << std::endl;

    // Write container to bytes
    Writer writer;
    container.write(writer);
    std::vector<uint8_t> data = writer.finish();

    std::cout << "Serialized to " << data.size() << " bytes" << std::endl;

    // Add padding bytes manually (since skip fields are not written but must exist in the binary)
    std::vector<uint8_t> complete_data;
    size_t pos = 0;

    // Write container data up to first entry's padding
    // Magic (4) + num_entries (2) = 6
    // Entry1: filename_len (1) + filename (8) + file_size (4) + file_data (13) + padding_size (2) = 28
    // Total before padding: 6 + 28 = 34
    size_t offset_entry1_padding = 6 + 1 + 8 + 4 + 13 + 2;
    complete_data.insert(complete_data.end(), data.begin(), data.begin() + offset_entry1_padding);
    // Add entry1 padding (3 bytes)
    complete_data.insert(complete_data.end(), 3, 0x00);

    // Entry2: filename_len (1) + filename (8) + file_size (4) + file_data (5) + padding_size (2) = 20
    size_t offset_entry2_start = offset_entry1_padding;
    size_t offset_entry2_padding = offset_entry2_start + 1 + 8 + 4 + 5 + 2;
    complete_data.insert(complete_data.end(), data.begin() + offset_entry1_padding, data.begin() + offset_entry2_padding);
    // No padding for entry2

    // Entry3: filename_len (1) + filename (9) + file_size (4) + file_data (0) + padding_size (2) = 16
    size_t offset_entry3_start = offset_entry2_padding;
    size_t offset_entry3_padding = offset_entry3_start + 1 + 9 + 4 + 0 + 2;
    complete_data.insert(complete_data.end(), data.begin() + offset_entry2_padding, data.begin() + offset_entry3_padding);
    // Add entry3 padding (16 bytes)
    complete_data.insert(complete_data.end(), 16, 0xFF);

    std::cout << "Complete binary with padding: " << complete_data.size() << " bytes" << std::endl;

    // Read container back
    Reader reader(complete_data);
    Container parsed = Container::read(reader);

    std::cout << "Parsed container with " << parsed.num_entries << " entries" << std::endl;

    // Verify magic
    assert(parsed.magic == 0x434E5452);
    std::cout << "✓ Magic number correct" << std::endl;

    // Verify num_entries
    assert(parsed.num_entries == 3);
    std::cout << "✓ Number of entries correct" << std::endl;

    // Verify entry 1
    assert(parsed.entries[0].filename_len == 8);
    assert(parsed.entries[0].filename == "test.txt");
    assert(parsed.entries[0].file_size == 13);
    assert(parsed.entries[0].file_data.size() == 13);
    assert(std::memcmp(parsed.entries[0].file_data.data(), entry1.file_data.data(), 13) == 0);
    assert(parsed.entries[0].padding_size == 3);
    std::cout << "✓ Entry 1 correct (filename: " << parsed.entries[0].filename << ")" << std::endl;

    // Verify entry 2
    assert(parsed.entries[1].filename_len == 8);
    assert(parsed.entries[1].filename == "data.bin");
    assert(parsed.entries[1].file_size == 5);
    assert(parsed.entries[1].file_data.size() == 5);
    assert(std::memcmp(parsed.entries[1].file_data.data(), entry2.file_data.data(), 5) == 0);
    assert(parsed.entries[1].padding_size == 0);
    std::cout << "✓ Entry 2 correct (filename: " << parsed.entries[1].filename << ")" << std::endl;

    // Verify entry 3
    assert(parsed.entries[2].filename_len == 9);
    assert(parsed.entries[2].filename == "empty.txt");
    assert(parsed.entries[2].file_size == 0);
    assert(parsed.entries[2].file_data.size() == 0);
    assert(parsed.entries[2].padding_size == 16);
    std::cout << "✓ Entry 3 correct (filename: " << parsed.entries[2].filename << ")" << std::endl;

    std::cout << std::endl << "All tests passed!" << std::endl;

    return 0;
}

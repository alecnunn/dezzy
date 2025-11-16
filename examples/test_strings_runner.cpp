#include "test_strings.hpp"
#include <iostream>
#include <vector>
#include <cassert>

using namespace teststrings;

int main() {
    std::cout << "=== Testing String Types ===\n\n";

    // Create test data
    FileHeader original;
    original.signature = "DEZZ";
    original.name_len = 8;
    original.filename = "test.dat";
    original.path = "/usr/local/bin";

    // Write to bytes
    Writer writer;
    original.write(writer);
    std::vector<uint8_t> data = writer.finish();

    std::cout << "Wrote " << data.size() << " bytes\n";
    std::cout << "  signature: \"" << original.signature << "\"\n";
    std::cout << "  name_len: " << (int)original.name_len << "\n";
    std::cout << "  filename: \"" << original.filename << "\"\n";
    std::cout << "  path: \"" << original.path << "\"\n\n";

    // Read it back
    Reader reader(data);
    FileHeader parsed = FileHeader::read(reader);

    std::cout << "Read back:\n";
    std::cout << "  signature: \"" << parsed.signature << "\"\n";
    std::cout << "  name_len: " << (int)parsed.name_len << "\n";
    std::cout << "  filename: \"" << parsed.filename << "\"\n";
    std::cout << "  path: \"" << parsed.path << "\"\n\n";

    // Verify
    assert(parsed.signature == original.signature);
    assert(parsed.name_len == original.name_len);
    assert(parsed.filename == original.filename);
    assert(parsed.path == original.path);

    std::cout << "=== All string tests passed! ===\n";
    return 0;
}

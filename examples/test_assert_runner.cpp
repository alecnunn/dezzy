#include "test_assert.hpp"
#include <iostream>
#include <vector>
#include <cstdint>
#include <cassert>

using namespace testassert;

void test_valid_header() {
    std::cout << "Test: Valid header... ";

    // Create valid header data: magic=[0x89, 0x50, 0x4E, 0x47], version=1, width=100, height=200, flags=3
    std::vector<uint8_t> data = {
        0x89, 0x50, 0x4E, 0x47,  // magic (PNG signature)
        0x00, 0x01,              // version = 1 (big endian)
        0x00, 0x00, 0x00, 0x64,  // width = 100 (big endian)
        0x00, 0x00, 0x00, 0xC8,  // height = 200 (big endian)
        0x03                      // flags = 3 (valid range 0-7)
    };

    Reader reader(data);
    Header header = Header::read(reader);

    assert(header.magic[0] == 0x89);
    assert(header.magic[1] == 0x50);
    assert(header.magic[2] == 0x4E);
    assert(header.magic[3] == 0x47);
    assert(header.version == 1);
    assert(header.width == 100);
    assert(header.height == 200);
    assert(header.flags == 3);

    std::cout << "PASSED\n";
}

void test_invalid_magic() {
    std::cout << "Test: Invalid magic number... ";

    // Invalid magic number
    std::vector<uint8_t> data = {
        0x00, 0x00, 0x00, 0x00,  // wrong magic
        0x00, 0x01,
        0x00, 0x00, 0x00, 0x64,
        0x00, 0x00, 0x00, 0xC8,
        0x03
    };

    Reader reader(data);

    try {
        Header::read(reader);
        std::cout << "FAILED (should have thrown)\n";
        std::exit(1);
    } catch (const ParseError& e) {
        std::string msg = e.what();
        assert(msg.find("magic") != std::string::npos);
        std::cout << "PASSED (caught: " << e.what() << ")\n";
    }
}

void test_invalid_version() {
    std::cout << "Test: Invalid version (must be >= 1)... ";

    // version = 0 (invalid)
    std::vector<uint8_t> data = {
        0x89, 0x50, 0x4E, 0x47,
        0x00, 0x00,              // version = 0 (invalid)
        0x00, 0x00, 0x00, 0x64,
        0x00, 0x00, 0x00, 0xC8,
        0x03
    };

    Reader reader(data);

    try {
        Header::read(reader);
        std::cout << "FAILED (should have thrown)\n";
        std::exit(1);
    } catch (const ParseError& e) {
        std::string msg = e.what();
        assert(msg.find("version") != std::string::npos);
        std::cout << "PASSED (caught: " << e.what() << ")\n";
    }
}

void test_invalid_width() {
    std::cout << "Test: Invalid width (must be > 0)... ";

    // width = 0 (invalid)
    std::vector<uint8_t> data = {
        0x89, 0x50, 0x4E, 0x47,
        0x00, 0x01,
        0x00, 0x00, 0x00, 0x00,  // width = 0 (invalid)
        0x00, 0x00, 0x00, 0xC8,
        0x03
    };

    Reader reader(data);

    try {
        Header::read(reader);
        std::cout << "FAILED (should have thrown)\n";
        std::exit(1);
    } catch (const ParseError& e) {
        std::string msg = e.what();
        assert(msg.find("width") != std::string::npos);
        std::cout << "PASSED (caught: " << e.what() << ")\n";
    }
}

void test_invalid_height() {
    std::cout << "Test: Invalid height (must be > 0)... ";

    // height = 0 (invalid)
    std::vector<uint8_t> data = {
        0x89, 0x50, 0x4E, 0x47,
        0x00, 0x01,
        0x00, 0x00, 0x00, 0x64,
        0x00, 0x00, 0x00, 0x00,  // height = 0 (invalid)
        0x03
    };

    Reader reader(data);

    try {
        Header::read(reader);
        std::cout << "FAILED (should have thrown)\n";
        std::exit(1);
    } catch (const ParseError& e) {
        std::string msg = e.what();
        assert(msg.find("height") != std::string::npos);
        std::cout << "PASSED (caught: " << e.what() << ")\n";
    }
}

void test_invalid_flags_too_high() {
    std::cout << "Test: Invalid flags (must be in range 0-7)... ";

    // flags = 8 (out of range)
    std::vector<uint8_t> data = {
        0x89, 0x50, 0x4E, 0x47,
        0x00, 0x01,
        0x00, 0x00, 0x00, 0x64,
        0x00, 0x00, 0x00, 0xC8,
        0x08                      // flags = 8 (invalid, must be 0-7)
    };

    Reader reader(data);

    try {
        Header::read(reader);
        std::cout << "FAILED (should have thrown)\n";
        std::exit(1);
    } catch (const ParseError& e) {
        std::string msg = e.what();
        assert(msg.find("flags") != std::string::npos);
        std::cout << "PASSED (caught: " << e.what() << ")\n";
    }
}

void test_roundtrip() {
    std::cout << "Test: Write/Read roundtrip... ";

    // Create a header
    Header original;
    original.magic = {0x89, 0x50, 0x4E, 0x47};
    original.version = 2;
    original.width = 1920;
    original.height = 1080;
    original.flags = 5;

    // Write it
    Writer writer;
    original.write(writer);
    std::vector<uint8_t> data = writer.finish();

    // Read it back
    Reader reader(data);
    Header read_back = Header::read(reader);

    // Verify
    assert(read_back.magic == original.magic);
    assert(read_back.version == original.version);
    assert(read_back.width == original.width);
    assert(read_back.height == original.height);
    assert(read_back.flags == original.flags);

    std::cout << "PASSED\n";
}

int main() {
    std::cout << "=== Testing Assertion Validation ===\n\n";

    test_valid_header();
    test_invalid_magic();
    test_invalid_version();
    test_invalid_width();
    test_invalid_height();
    test_invalid_flags_too_high();
    test_roundtrip();

    std::cout << "\n=== All tests passed! ===\n";
    return 0;
}

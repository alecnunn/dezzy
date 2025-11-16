#include "generated/nestedformat.hpp"
#include <iostream>
#include <cassert>

int main() {
    using namespace nestedformat;

    // Create a test document
    Document doc;
    doc.version = 1;
    doc.count = 5;
    doc.bounds.top_left.x = 10;
    doc.bounds.top_left.y = 20;
    doc.bounds.bottom_right.x = 100;
    doc.bounds.bottom_right.y = 200;
    doc.bounds.color = 0xFF0000FF; // Red

    // Serialize it
    Writer writer;
    doc.write(writer);
    std::vector<uint8_t> data = writer.finish();

    std::cout << "Serialized " << data.size() << " bytes" << std::endl;

    // Deserialize it
    Reader reader(std::span<const uint8_t>(data.data(), data.size()));
    Document doc2 = Document::read(reader);

    // Verify
    assert(doc2.version == 1);
    assert(doc2.count == 5);
    assert(doc2.bounds.top_left.x == 10);
    assert(doc2.bounds.top_left.y == 20);
    assert(doc2.bounds.bottom_right.x == 100);
    assert(doc2.bounds.bottom_right.y == 200);
    assert(doc2.bounds.color == 0xFF0000FF);

    std::cout << "All tests passed!" << std::endl;
    std::cout << "Document version: " << doc2.version << std::endl;
    std::cout << "Bounds: (" << doc2.bounds.top_left.x << ", " << doc2.bounds.top_left.y
              << ") to (" << doc2.bounds.bottom_right.x << ", " << doc2.bounds.bottom_right.y << ")" << std::endl;

    return 0;
}

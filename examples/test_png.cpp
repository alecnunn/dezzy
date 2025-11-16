#include "png.hpp"
#include <iostream>
#include <vector>
#include <cassert>

using namespace png;

int main() {
    std::cout << "Testing PNG roundtrip...\n";

    // Create a minimal PNG with IHDR and IEND chunks
    PNG png;

    // PNG signature
    png.signature = {137, 80, 78, 71, 13, 10, 26, 10};

    // Create IHDR chunk
    Chunk ihdr;
    ihdr.length = 13;
    ihdr.chunk_type = {73, 72, 68, 82};  // 'IHDR'
    ihdr.data.resize(13);
    // Width: 1, Height: 1, Bit depth: 8, Color type: 2 (RGB)
    ihdr.data[0] = 0; ihdr.data[1] = 0; ihdr.data[2] = 0; ihdr.data[3] = 1;
    ihdr.data[4] = 0; ihdr.data[5] = 0; ihdr.data[6] = 0; ihdr.data[7] = 1;
    ihdr.data[8] = 8; ihdr.data[9] = 2; ihdr.data[10] = 0;
    ihdr.data[11] = 0; ihdr.data[12] = 0;
    ihdr.crc = 0;  // Simplified

    // Create IEND chunk
    Chunk iend;
    iend.length = 0;
    iend.chunk_type = {73, 69, 78, 68};  // 'IEND'
    iend.crc = 0;

    png.chunks.push_back(ihdr);
    png.chunks.push_back(iend);

    // Write to bytes
    Writer writer;
    png.write(writer);
    auto png_bytes = writer.finish();
    std::cout << "Generated PNG: " << png_bytes.size() << " bytes\n";

    // Read it back
    Reader reader(png_bytes);
    PNG parsed_png = PNG::read(reader);
    std::cout << "Parsed PNG: " << reader.position() << " bytes read\n";

    // Verify signature
    assert(parsed_png.signature == png.signature && "Signature mismatch");

    // Verify chunk count (critical test - until-condition should stop at IEND)
    assert(parsed_png.chunks.size() == 2 && "Expected 2 chunks");

    // Verify IHDR
    assert(parsed_png.chunks[0].chunk_type[0] == 73 &&
           parsed_png.chunks[0].chunk_type[1] == 72 &&
           parsed_png.chunks[0].chunk_type[2] == 68 &&
           parsed_png.chunks[0].chunk_type[3] == 82 &&
           "First chunk should be IHDR");
    assert(parsed_png.chunks[0].length == 13 && "IHDR length mismatch");

    // Verify IEND (this is the critical test)
    assert(parsed_png.chunks[1].chunk_type[0] == 73 &&
           parsed_png.chunks[1].chunk_type[1] == 69 &&
           parsed_png.chunks[1].chunk_type[2] == 78 &&
           parsed_png.chunks[1].chunk_type[3] == 68 &&
           "Last chunk should be IEND");
    assert(parsed_png.chunks[1].length == 0 && "IEND should have no data");

    std::cout << "[OK] PNG signature correct\n";
    std::cout << "[OK] Chunk count correct (until-condition stopped at IEND)\n";
    std::cout << "[OK] IHDR chunk parsed correctly\n";
    std::cout << "[OK] IEND chunk parsed correctly\n";
    std::cout << "\nAll tests passed!\n";

    return 0;
}

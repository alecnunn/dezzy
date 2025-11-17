# Real-World Format Testing

This document describes the real-world binary formats we've successfully tested with dezzy.

## PNG Format

### Format Definition
`examples/png.yaml` - Complete PNG file format with chunks

### Features Used
- **Big-endian** byte order
- **Fixed arrays**: 8-byte signature, 4-byte chunk types
- **Dynamic arrays**: `u8[length]` for variable-length chunk data
- **Until-condition arrays**: `Chunk[]` reading until IEND chunk
- **Nested structures**: Chunk struct inside PNG struct

### Test Files
- `examples/logo.png` - Real 71KB PNG file (640x360 pixels)
- Generated with: Standard PNG encoder

### Test Results ✓
Successfully parsed real PNG file with:
- Correct PNG signature validation
- 12 chunks of various types (IHDR, gAMA, cHRM, bKGD, pHYs, tIME, IDAT×3, tEXt×2, IEND)
- Variable-length data fields (32KB+ IDAT chunks)
- Until-condition correctly stopped at IEND chunk
- All 72,151 bytes parsed without errors

### Code Generation
**C++ Output:**
- `Chunk` struct with `std::vector<uint8_t>` for dynamic data
- `PNG` struct with `std::vector<Chunk>` for chunk array
- Type-safe read/write operations
- Proper endianness handling (big-endian for PNG)

**Test Program:** `examples/test_real_png.cpp`
- Reads real PNG file from disk
- Displays signature, chunk count, and chunk details
- Extracts and displays image dimensions from IHDR
- Validates PNG structure

### Validated Features
1. ✓ Variable-length arrays sized by previous field
2. ✓ Until-condition parsing (stop when condition met)
3. ✓ Big-endian integer parsing
4. ✓ Nested struct arrays
5. ✓ Multiple instances of same chunk type (IDAT, tEXt)

## ZIP Format

### Format Definition
`examples/zip.yaml` - ZIP archive structures

### Features Used
- **Little-endian** byte order
- **Assertions**: Signature validation with `assert: { equals: 0x... }`
- **Dynamic arrays**: Variable-length filenames, extra fields, comments
- **Complex structures**: Local file headers, central directory headers, EOCD

### Test Files
- `examples/test.zip` - Generated 395-byte ZIP with 3 files
- Generated with: Python `zipfile` module (standard ZIP format)

### Test Results ✓
Successfully parsed ZIP file:
- Located End of Central Directory record (offset 373)
- Parsed EOCD: 3 entries, 186-byte central directory at offset 187
- Parsed Central Directory Header for "hello.txt"
- Validated signature assertions (0x06054b50 for EOCD)
- Variable-length fields: filename, extra fields, comments

### Code Generation
**C++ Output:**
- `LocalFileHeader`, `CentralDirectoryHeader`, `EndOfCentralDirectory` structs
- Dynamic arrays with `std::vector<uint8_t>`
- Signature validation via assertions
- Runtime ParseError exceptions for invalid signatures

**Test Program:** `examples/test_zip.cpp`
- Searches for EOCD signature from end of file
- Parses EOCD record
- Parses first Central Directory entry
- Displays file metadata (compression, sizes, filename)

### Validated Features
1. ✓ Little-endian integer parsing
2. ✓ Assertion-based validation (signature checks)
3. ✓ Multiple variable-length fields in single struct
4. ✓ Complex multi-structure formats
5. ✓ Runtime error handling for invalid data

## Summary Statistics

### Formats Tested: 2
1. PNG - Image format (big-endian, streaming chunks)
2. ZIP - Archive format (little-endian, complex directory structure)

### Features Validated
- ✓ Both endianness modes (big and little)
- ✓ Variable-length arrays (`type[size_field]`)
- ✓ Fixed-size arrays (`type[N]`)
- ✓ Until-condition arrays
- ✓ Nested structures
- ✓ Assertions for validation
- ✓ Runtime error handling
- ✓ Multi-kilobyte file parsing

### File Sizes Tested
- Smallest: 395 bytes (ZIP)
- Largest: 72,151 bytes (PNG)
- Total test data: ~72KB across formats

### Generated Code Quality
- **Type safety**: No raw pointers, uses STL containers
- **Error handling**: Proper exceptions with descriptive messages
- **Readability**: Clean, idiomatic C++20 code
- **Performance**: Single-pass parsing, minimal allocations

## Future Real-World Formats

Potential formats to add:
- **ELF**: Executable and Linkable Format (binary executables)
- **PE**: Portable Executable (Windows executables)
- **WAV**: Waveform Audio File Format (RIFF-based)
- **MP3**: MPEG audio with ID3 tags
- **SQLite**: Database file format
- **Protocol Buffers**: Binary serialization format
- **Bitcoin blocks**: Cryptocurrency blockchain data

Each format would validate different combinations of features and edge cases.

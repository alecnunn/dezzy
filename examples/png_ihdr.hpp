#pragma once

#include <cstdint>
#include <array>
#include <vector>
#include <span>
#include <optional>
#include <string>
#include <stdexcept>
#include <cstring>

namespace pngheader {

class ParseError : public std::runtime_error {
public:
    explicit ParseError(const std::string& message)
        : std::runtime_error(message) {}
};

class Reader {
public:
    explicit Reader(std::span<const uint8_t> data)
        : data_(data), position_(0) {}

    template<typename T>
    T read_le() {
        if (position_ + sizeof(T) > data_.size()) {
            throw ParseError("Unexpected end of data");
        }
        T value;
        std::memcpy(&value, &data_[position_], sizeof(T));
        position_ += sizeof(T);
        return value;
    }

    template<typename T>
    T read_be() {
        if (position_ + sizeof(T) > data_.size()) {
            throw ParseError("Unexpected end of data");
        }
        T value = 0;
        for (size_t i = 0; i < sizeof(T); ++i) {
            value = (value << 8) | data_[position_ + i];
        }
        position_ += sizeof(T);
        return value;
    }

    size_t position() const { return position_; }
    size_t remaining() const { return data_.size() - position_; }

private:
    std::span<const uint8_t> data_;
    size_t position_;
};

class Writer {
public:
    template<typename T>
    void write_le(T value) {
        uint8_t bytes[sizeof(T)];
        std::memcpy(bytes, &value, sizeof(T));
        data_.insert(data_.end(), bytes, bytes + sizeof(T));
    }

    template<typename T>
    void write_be(T value) {
        for (size_t i = sizeof(T); i > 0; --i) {
            data_.push_back(static_cast<uint8_t>((value >> ((i - 1) * 8)) & 0xFF));
        }
    }

    std::vector<uint8_t> finish() { return std::move(data_); }

private:
    std::vector<uint8_t> data_;
};

enum class ColorType : uint8_t {
    GRAYSCALE = 0,
    RGB = 2,
    PALETTE = 3,
    GRAYSCALE_ALPHA = 4,
    RGBA = 6
};

enum class CompressionMethod : uint8_t {
    DEFLATE = 0
};

enum class FilterMethod : uint8_t {
    ADAPTIVE = 0
};

enum class InterlaceMethod : uint8_t {
    NONE = 0,
    ADAM7 = 1
};

struct IHDRChunk {
    uint32_t width;
    uint32_t height;
    uint8_t bit_depth;
    ColorType color_type;
    CompressionMethod compression_method;
    FilterMethod filter_method;
    InterlaceMethod interlace_method;

    static IHDRChunk read(Reader& reader);
    void write(Writer& writer) const;
};

inline IHDRChunk IHDRChunk::read(Reader& reader) {
    IHDRChunk result;
    result.width = reader.read_be<uint32_t>();
    result.height = reader.read_be<uint32_t>();
    result.bit_depth = reader.read_le<uint8_t>();
    result.color_type = static_cast<ColorType>(reader.read_le<uint8_t>());
    result.compression_method = static_cast<CompressionMethod>(reader.read_le<uint8_t>());
    result.filter_method = static_cast<FilterMethod>(reader.read_le<uint8_t>());
    result.interlace_method = static_cast<InterlaceMethod>(reader.read_le<uint8_t>());
    return result;
}

inline void IHDRChunk::write(Writer& writer) const {
    writer.write_be(width);
    writer.write_be(height);
    writer.write_le(bit_depth);
    writer.write_le(color_type);
    writer.write_le(compression_method);
    writer.write_le(filter_method);
    writer.write_le(interlace_method);
}

struct Chunk {
    uint32_t length;
    std::array<uint8_t, 4> chunk_type;
    std::vector<uint8_t> data;
    uint32_t crc;

    static Chunk read(Reader& reader);
    void write(Writer& writer) const;
};

inline Chunk Chunk::read(Reader& reader) {
    Chunk result;
    result.length = reader.read_be<uint32_t>();
    for (size_t i = 0; i < 4; ++i) {
        result.chunk_type[i] = reader.read_le<uint8_t>();
    }
    result.data.resize(result.length);
    for (size_t i = 0; i < result.length; ++i) {
        result.data[i] = reader.read_le<uint8_t>();
    }
    result.crc = reader.read_be<uint32_t>();
    return result;
}

inline void Chunk::write(Writer& writer) const {
    writer.write_be(length);
    for (size_t i = 0; i < 4; ++i) {
        writer.write_le(chunk_type[i]);
    }
    for (size_t i = 0; i < length; ++i) {
        writer.write_le(data[i]);
    }
    writer.write_be(crc);
}

struct PNGWithIHDR {
    std::array<uint8_t, 8> signature;
    uint32_t ihdr_length;
    std::array<uint8_t, 4> ihdr_type;
    IHDRChunk ihdr;
    uint32_t ihdr_crc;
    std::vector<Chunk> remaining_chunks;

    static PNGWithIHDR read(Reader& reader);
    void write(Writer& writer) const;
};

inline PNGWithIHDR PNGWithIHDR::read(Reader& reader) {
    PNGWithIHDR result;
    for (size_t i = 0; i < 8; ++i) {
        result.signature[i] = reader.read_le<uint8_t>();
    }
    result.ihdr_length = reader.read_be<uint32_t>();
    for (size_t i = 0; i < 4; ++i) {
        result.ihdr_type[i] = reader.read_le<uint8_t>();
    }
    result.ihdr = IHDRChunk::read(reader);
    result.ihdr_crc = reader.read_be<uint32_t>();
    while (reader.remaining() > 0) {
        result.remaining_chunks.push_back(Chunk::read(reader));
    }
    return result;
}

inline void PNGWithIHDR::write(Writer& writer) const {
    for (size_t i = 0; i < 8; ++i) {
        writer.write_le(signature[i]);
    }
    writer.write_be(ihdr_length);
    for (size_t i = 0; i < 4; ++i) {
        writer.write_le(ihdr_type[i]);
    }
    ihdr.write(writer);
    writer.write_be(ihdr_crc);
    for (size_t i = 0; i < remaining_chunks.size(); ++i) {
        remaining_chunks[i].write(writer);
    }
}


} // namespace pngheader

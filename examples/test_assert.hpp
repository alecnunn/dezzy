#pragma once

#include <cstdint>
#include <array>
#include <vector>
#include <span>
#include <optional>
#include <string>
#include <stdexcept>
#include <cstring>

namespace testassert {

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

struct Header {
    std::array<uint8_t, 4> magic;
    uint16_t version;
    uint32_t width;
    uint32_t height;
    uint8_t flags;

    static Header read(Reader& reader);
    void write(Writer& writer) const;
};

inline Header Header::read(Reader& reader) {
    Header result;
    for (size_t i = 0; i < 4; ++i) {
        result.magic[i] = reader.read_le<uint8_t>();
    }
    result.version = reader.read_be<uint16_t>();
    result.width = reader.read_be<uint32_t>();
    result.height = reader.read_be<uint32_t>();
    result.flags = reader.read_le<uint8_t>();
    return result;
}

inline void Header::write(Writer& writer) const {
    for (size_t i = 0; i < 4; ++i) {
        writer.write_le(magic[i]);
    }
    writer.write_be(version);
    writer.write_be(width);
    writer.write_be(height);
    writer.write_le(flags);
}


} // namespace testassert

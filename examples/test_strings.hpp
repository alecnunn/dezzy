#pragma once

#include <cstdint>
#include <array>
#include <vector>
#include <span>
#include <optional>
#include <string>
#include <stdexcept>
#include <cstring>

namespace teststrings {

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

struct FileHeader {
    std::string signature;
    uint8_t name_len;
    std::string filename;
    std::string path;

    static FileHeader read(Reader& reader);
    void write(Writer& writer) const;
};

inline FileHeader FileHeader::read(Reader& reader) {
    FileHeader result;
    {
        std::vector<uint8_t> bytes(4);
        for (size_t i = 0; i < 4; ++i) {
            bytes[i] = reader.read_le<uint8_t>();
        }
        result.signature = std::string(reinterpret_cast<const char*>(bytes.data()), bytes.size());
    }
    result.name_len = reader.read_le<uint8_t>();
    {
        std::vector<uint8_t> bytes(result.name_len);
        for (size_t i = 0; i < result.name_len; ++i) {
            bytes[i] = reader.read_le<uint8_t>();
        }
        result.filename = std::string(reinterpret_cast<const char*>(bytes.data()), bytes.size());
    }
    {
        std::vector<uint8_t> bytes;
        uint8_t byte;
        while ((byte = reader.read_le<uint8_t>()) != 0) {
            bytes.push_back(byte);
        }
        result.path = std::string(reinterpret_cast<const char*>(bytes.data()), bytes.size());
    }
    return result;
}

inline void FileHeader::write(Writer& writer) const {
    for (size_t i = 0; i < 4; ++i) {
        writer.write_le(static_cast<uint8_t>(signature.size() > i ? signature[i] : 0));
    }
    writer.write_le(name_len);
    for (size_t i = 0; i < filename.size(); ++i) {
        writer.write_le(static_cast<uint8_t>(filename[i]));
    }
    for (size_t i = 0; i < path.size(); ++i) {
        writer.write_le(static_cast<uint8_t>(path[i]));
    }
    writer.write_le(static_cast<uint8_t>(0));  // null terminator
}


} // namespace teststrings

#pragma once

#include <cstdint>
#include <array>
#include <vector>
#include <span>
#include <optional>
#include <string>
#include <stdexcept>
#include <cstring>

namespace testenum {

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

enum class Status : uint8_t {
    OK = 0,
    ERROR = 1,
    PENDING = 2
};

struct Message {
    Status status;
    uint32_t value;

    static Message read(Reader& reader);
    void write(Writer& writer) const;
};

inline Message Message::read(Reader& reader) {
    Message result;
    result.status = static_cast<Status>(reader.read_le<uint8_t>());
    result.value = reader.read_be<uint32_t>();
    return result;
}

inline void Message::write(Writer& writer) const {
    writer.write_le(status);
    writer.write_be(value);
}


} // namespace testenum

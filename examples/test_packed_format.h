#pragma once

#include <cstdint>
#include <array>
#include <vector>
#include <span>
#include <optional>
#include <string>
#include <stdexcept>
#include <cstring>

namespace packedformat {

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

    void skip(size_t bytes) {
        if (position_ + bytes > data_.size()) {
            throw ParseError("Unexpected end of data during skip");
        }
        position_ += bytes;
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

    void write_padding(size_t bytes) {
        data_.insert(data_.end(), bytes, 0);
    }

    void align(size_t boundary) {
        size_t padding = (boundary - (data_.size() % boundary)) % boundary;
        write_padding(padding);
    }

    size_t position() const { return data_.size(); }

    std::vector<uint8_t> finish() { return std::move(data_); }

private:
    std::vector<uint8_t> data_;
};

class BitReader {
public:
    explicit BitReader(Reader& reader) : reader_(reader), current_byte_(0), bits_remaining_(0) {}

    uint32_t read_bits_msb(size_t num_bits) {
        uint32_t result = 0;
        while (num_bits > 0) {
            if (bits_remaining_ == 0) {
                current_byte_ = reader_.read_le<uint8_t>();
                bits_remaining_ = 8;
            }
            size_t bits_to_read = std::min(num_bits, bits_remaining_);
            result = (result << bits_to_read) |
                     ((current_byte_ >> (bits_remaining_ - bits_to_read)) & ((1 << bits_to_read) - 1));
            bits_remaining_ -= bits_to_read;
            num_bits -= bits_to_read;
        }
        return result;
    }

    int32_t read_signed_bits_msb(size_t num_bits) {
        uint32_t unsigned_value = read_bits_msb(num_bits);
        // Sign extend if high bit is set
        if (unsigned_value & (1 << (num_bits - 1))) {
            // Set all bits above num_bits to 1
            uint32_t sign_extend_mask = ~((1 << num_bits) - 1);
            return static_cast<int32_t>(unsigned_value | sign_extend_mask);
        }
        return static_cast<int32_t>(unsigned_value);
    }

private:
    Reader& reader_;
    uint8_t current_byte_;
    size_t bits_remaining_;
};

class BitWriter {
public:
    explicit BitWriter(Writer& writer) : writer_(writer), current_byte_(0), bits_used_(0) {}

    void write_bits_msb(uint32_t value, size_t num_bits) {
        while (num_bits > 0) {
            size_t bits_to_write = std::min(num_bits, 8 - bits_used_);
            uint8_t bits = (value >> (num_bits - bits_to_write)) & ((1 << bits_to_write) - 1);
            current_byte_ = (current_byte_ << bits_to_write) | bits;
            bits_used_ += bits_to_write;
            num_bits -= bits_to_write;

            if (bits_used_ == 8) {
                writer_.write_le(current_byte_);
                current_byte_ = 0;
                bits_used_ = 0;
            }
        }
    }

    void flush() {
        if (bits_used_ > 0) {
            // Pad remaining bits with zeros
            current_byte_ <<= (8 - bits_used_);
            writer_.write_le(current_byte_);
            current_byte_ = 0;
            bits_used_ = 0;
        }
    }

    ~BitWriter() {
        // Auto-flush on destruction if needed
        if (bits_used_ > 0) {
            flush();
        }
    }

private:
    Writer& writer_;
    uint8_t current_byte_;
    size_t bits_used_;
};

struct PackedHeader {
    uint32_t magic;
    uint8_t version;
    uint8_t compressed;
    uint8_t encrypted;
    uint8_t reserved_bits;
    uint32_t data_size;
    uint64_t data_offset;
    uint8_t priority;
    int8_t status;
    uint8_t flags;
    uint32_t checksum;

    static PackedHeader read(Reader& reader);
    void write(Writer& writer) const;
};

inline PackedHeader PackedHeader::read(Reader& reader) {
    PackedHeader result;
    static BitReader bit_reader(reader);
    result.magic = reader.read_le<uint32_t>();
    if (result.magic != 1346456388) {
        throw ParseError("Field 'magic' must equal 1346456388, got " + std::to_string(result.magic));
    }
    result.version = bit_reader.read_bits_msb(3);
    result.compressed = bit_reader.read_bits_msb(1);
    result.encrypted = bit_reader.read_bits_msb(1);
    result.reserved_bits = bit_reader.read_bits_msb(3);
    reader.skip(2);
    result.data_size = reader.read_le<uint32_t>();
    {
        size_t padding = (8 - (reader.position() % 8)) % 8;
        reader.skip(padding);
    }
    result.data_offset = reader.read_le<uint64_t>();
    result.priority = bit_reader.read_bits_msb(2);
    result.status = bit_reader.read_signed_bits_msb(3);
    result.flags = bit_reader.read_bits_msb(3);
    {
        size_t padding = (4 - (reader.position() % 4)) % 4;
        reader.skip(padding);
    }
    result.checksum = reader.read_le<uint32_t>();
    return result;
}

inline void PackedHeader::write(Writer& writer) const {
    static BitWriter bit_writer(writer);
    writer.write_le(magic);
    bit_writer.write_bits_msb(version, 3);
    bit_writer.write_bits_msb(compressed, 1);
    bit_writer.write_bits_msb(encrypted, 1);
    bit_writer.write_bits_msb(reserved_bits, 3);
    writer.write_padding(2);
    writer.write_le(data_size);
    writer.align(8);
    writer.write_le(data_offset);
    bit_writer.write_bits_msb(priority, 2);
    bit_writer.write_bits_msb(status, 3);
    bit_writer.write_bits_msb(flags, 3);
    writer.align(4);
    writer.write_le(checksum);
}


} // namespace packedformat

#pragma once

#include <cstdint>
#include <array>
#include <vector>
#include <span>
#include <optional>
#include <string>
#include <stdexcept>
#include <cstring>

namespace zip {

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

struct CentralDirectoryHeader {
    uint32_t signature;
    uint16_t version_made_by;
    uint16_t version_needed;
    uint16_t flags;
    uint16_t compression_method;
    uint16_t last_mod_time;
    uint16_t last_mod_date;
    uint32_t crc32;
    uint32_t compressed_size;
    uint32_t uncompressed_size;
    uint16_t filename_length;
    uint16_t extra_field_length;
    uint16_t comment_length;
    uint16_t disk_number_start;
    uint16_t internal_attrs;
    uint32_t external_attrs;
    uint32_t local_header_offset;
    std::vector<uint8_t> filename;
    std::vector<uint8_t> extra_field;
    std::vector<uint8_t> comment;

    static CentralDirectoryHeader read(Reader& reader);
    void write(Writer& writer) const;
};

inline CentralDirectoryHeader CentralDirectoryHeader::read(Reader& reader) {
    CentralDirectoryHeader result;
    result.signature = reader.read_le<uint32_t>();
    if (result.signature != 33639248) {
        throw ParseError("Field 'signature' must equal 33639248, got " + std::to_string(result.signature));
    }
    result.version_made_by = reader.read_le<uint16_t>();
    result.version_needed = reader.read_le<uint16_t>();
    result.flags = reader.read_le<uint16_t>();
    result.compression_method = reader.read_le<uint16_t>();
    result.last_mod_time = reader.read_le<uint16_t>();
    result.last_mod_date = reader.read_le<uint16_t>();
    result.crc32 = reader.read_le<uint32_t>();
    result.compressed_size = reader.read_le<uint32_t>();
    result.uncompressed_size = reader.read_le<uint32_t>();
    result.filename_length = reader.read_le<uint16_t>();
    result.extra_field_length = reader.read_le<uint16_t>();
    result.comment_length = reader.read_le<uint16_t>();
    result.disk_number_start = reader.read_le<uint16_t>();
    result.internal_attrs = reader.read_le<uint16_t>();
    result.external_attrs = reader.read_le<uint32_t>();
    result.local_header_offset = reader.read_le<uint32_t>();
    result.filename.resize(result.filename_length);
    for (size_t i = 0; i < result.filename_length; ++i) {
        result.filename[i] = reader.read_le<uint8_t>();
    }
    result.extra_field.resize(result.extra_field_length);
    for (size_t i = 0; i < result.extra_field_length; ++i) {
        result.extra_field[i] = reader.read_le<uint8_t>();
    }
    result.comment.resize(result.comment_length);
    for (size_t i = 0; i < result.comment_length; ++i) {
        result.comment[i] = reader.read_le<uint8_t>();
    }
    return result;
}

inline void CentralDirectoryHeader::write(Writer& writer) const {
    writer.write_le(signature);
    writer.write_le(version_made_by);
    writer.write_le(version_needed);
    writer.write_le(flags);
    writer.write_le(compression_method);
    writer.write_le(last_mod_time);
    writer.write_le(last_mod_date);
    writer.write_le(crc32);
    writer.write_le(compressed_size);
    writer.write_le(uncompressed_size);
    writer.write_le(filename_length);
    writer.write_le(extra_field_length);
    writer.write_le(comment_length);
    writer.write_le(disk_number_start);
    writer.write_le(internal_attrs);
    writer.write_le(external_attrs);
    writer.write_le(local_header_offset);
    for (size_t i = 0; i < filename_length; ++i) {
        writer.write_le(filename[i]);
    }
    for (size_t i = 0; i < extra_field_length; ++i) {
        writer.write_le(extra_field[i]);
    }
    for (size_t i = 0; i < comment_length; ++i) {
        writer.write_le(comment[i]);
    }
}

struct EndOfCentralDirectory {
    uint32_t signature;
    uint16_t disk_number;
    uint16_t disk_with_cd;
    uint16_t num_entries_this_disk;
    uint16_t num_entries_total;
    uint32_t cd_size;
    uint32_t cd_offset;
    uint16_t comment_length;
    std::vector<uint8_t> comment;

    static EndOfCentralDirectory read(Reader& reader);
    void write(Writer& writer) const;
};

inline EndOfCentralDirectory EndOfCentralDirectory::read(Reader& reader) {
    EndOfCentralDirectory result;
    result.signature = reader.read_le<uint32_t>();
    if (result.signature != 101010256) {
        throw ParseError("Field 'signature' must equal 101010256, got " + std::to_string(result.signature));
    }
    result.disk_number = reader.read_le<uint16_t>();
    result.disk_with_cd = reader.read_le<uint16_t>();
    result.num_entries_this_disk = reader.read_le<uint16_t>();
    result.num_entries_total = reader.read_le<uint16_t>();
    result.cd_size = reader.read_le<uint32_t>();
    result.cd_offset = reader.read_le<uint32_t>();
    result.comment_length = reader.read_le<uint16_t>();
    result.comment.resize(result.comment_length);
    for (size_t i = 0; i < result.comment_length; ++i) {
        result.comment[i] = reader.read_le<uint8_t>();
    }
    return result;
}

inline void EndOfCentralDirectory::write(Writer& writer) const {
    writer.write_le(signature);
    writer.write_le(disk_number);
    writer.write_le(disk_with_cd);
    writer.write_le(num_entries_this_disk);
    writer.write_le(num_entries_total);
    writer.write_le(cd_size);
    writer.write_le(cd_offset);
    writer.write_le(comment_length);
    for (size_t i = 0; i < comment_length; ++i) {
        writer.write_le(comment[i]);
    }
}

struct LocalFileHeader {
    uint32_t signature;
    uint16_t version_needed;
    uint16_t flags;
    uint16_t compression_method;
    uint16_t last_mod_time;
    uint16_t last_mod_date;
    uint32_t crc32;
    uint32_t compressed_size;
    uint32_t uncompressed_size;
    uint16_t filename_length;
    uint16_t extra_field_length;
    std::vector<uint8_t> filename;
    std::vector<uint8_t> extra_field;

    static LocalFileHeader read(Reader& reader);
    void write(Writer& writer) const;
};

inline LocalFileHeader LocalFileHeader::read(Reader& reader) {
    LocalFileHeader result;
    result.signature = reader.read_le<uint32_t>();
    if (result.signature != 67324752) {
        throw ParseError("Field 'signature' must equal 67324752, got " + std::to_string(result.signature));
    }
    result.version_needed = reader.read_le<uint16_t>();
    result.flags = reader.read_le<uint16_t>();
    result.compression_method = reader.read_le<uint16_t>();
    result.last_mod_time = reader.read_le<uint16_t>();
    result.last_mod_date = reader.read_le<uint16_t>();
    result.crc32 = reader.read_le<uint32_t>();
    result.compressed_size = reader.read_le<uint32_t>();
    result.uncompressed_size = reader.read_le<uint32_t>();
    result.filename_length = reader.read_le<uint16_t>();
    result.extra_field_length = reader.read_le<uint16_t>();
    result.filename.resize(result.filename_length);
    for (size_t i = 0; i < result.filename_length; ++i) {
        result.filename[i] = reader.read_le<uint8_t>();
    }
    result.extra_field.resize(result.extra_field_length);
    for (size_t i = 0; i < result.extra_field_length; ++i) {
        result.extra_field[i] = reader.read_le<uint8_t>();
    }
    return result;
}

inline void LocalFileHeader::write(Writer& writer) const {
    writer.write_le(signature);
    writer.write_le(version_needed);
    writer.write_le(flags);
    writer.write_le(compression_method);
    writer.write_le(last_mod_time);
    writer.write_le(last_mod_date);
    writer.write_le(crc32);
    writer.write_le(compressed_size);
    writer.write_le(uncompressed_size);
    writer.write_le(filename_length);
    writer.write_le(extra_field_length);
    for (size_t i = 0; i < filename_length; ++i) {
        writer.write_le(filename[i]);
    }
    for (size_t i = 0; i < extra_field_length; ++i) {
        writer.write_le(extra_field[i]);
    }
}


} // namespace zip

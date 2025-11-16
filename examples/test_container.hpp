#pragma once

#include <cstdint>
#include <array>
#include <vector>
#include <span>
#include <optional>
#include <string>
#include <stdexcept>
#include <cstring>

namespace testcontainer {

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

    std::vector<uint8_t> finish() { return std::move(data_); }

private:
    std::vector<uint8_t> data_;
};

struct FileEntry {
    uint8_t filename_len;
    std::string filename;
    uint32_t file_size;
    std::vector<uint8_t> file_data;
    uint16_t padding_size;

    static FileEntry read(Reader& reader);
    void write(Writer& writer) const;
};

inline FileEntry FileEntry::read(Reader& reader) {
    FileEntry result;
    result.filename_len = reader.read_le<uint8_t>();
    {
        std::vector<uint8_t> bytes(result.filename_len);
        for (size_t i = 0; i < result.filename_len; ++i) {
            bytes[i] = reader.read_le<uint8_t>();
        }
        result.filename = std::string(reinterpret_cast<const char*>(bytes.data()), bytes.size());
    }
    result.file_size = reader.read_le<uint32_t>();
    result.file_data.resize(result.file_size);
    for (size_t i = 0; i < result.file_size; ++i) {
        result.file_data[i] = reader.read_le<uint8_t>();
    }
    result.padding_size = reader.read_le<uint16_t>();
    reader.skip(result.padding_size);
    return result;
}

inline void FileEntry::write(Writer& writer) const {
    writer.write_le(filename_len);
    for (size_t i = 0; i < filename.size(); ++i) {
        writer.write_le(static_cast<uint8_t>(filename[i]));
    }
    writer.write_le(file_size);
    for (size_t i = 0; i < file_data.size(); ++i) {
        writer.write_le(file_data[i]);
    }
    writer.write_le(padding_size);
}

struct Container {
    uint32_t magic;
    uint16_t num_entries;
    std::vector<FileEntry> entries;

    static Container read(Reader& reader);
    void write(Writer& writer) const;
};

inline Container Container::read(Reader& reader) {
    Container result;
    result.magic = reader.read_le<uint32_t>();
    if (result.magic != 1129206866) {
        throw ParseError("Field 'magic' must equal 1129206866, got " + std::to_string(result.magic));
    }
    result.num_entries = reader.read_le<uint16_t>();
    result.entries.resize(result.num_entries);
    for (size_t i = 0; i < result.num_entries; ++i) {
        result.entries[i] = FileEntry::read(reader);
    }
    return result;
}

inline void Container::write(Writer& writer) const {
    writer.write_le(magic);
    writer.write_le(num_entries);
    for (size_t i = 0; i < num_entries; ++i) {
        entries[i].write(writer);
    }
}


} // namespace testcontainer

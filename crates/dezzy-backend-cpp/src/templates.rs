pub fn generate_header_start(namespace: &str) -> String {
    format!(
        r#"#pragma once

#include <cstdint>
#include <array>
#include <vector>
#include <span>
#include <optional>
#include <string>
#include <stdexcept>
#include <cstring>

namespace {} {{

class ParseError : public std::runtime_error {{
public:
    explicit ParseError(const std::string& message)
        : std::runtime_error(message) {{}}
}};

class Reader {{
public:
    explicit Reader(std::span<const uint8_t> data)
        : data_(data), position_(0) {{}}

    template<typename T>
    T read_le() {{
        if (position_ + sizeof(T) > data_.size()) {{
            throw ParseError("Unexpected end of data");
        }}
        T value;
        std::memcpy(&value, &data_[position_], sizeof(T));
        position_ += sizeof(T);
        return value;
    }}

    template<typename T>
    T read_be() {{
        if (position_ + sizeof(T) > data_.size()) {{
            throw ParseError("Unexpected end of data");
        }}
        T value = 0;
        for (size_t i = 0; i < sizeof(T); ++i) {{
            value = (value << 8) | data_[position_ + i];
        }}
        position_ += sizeof(T);
        return value;
    }}

    void skip(size_t bytes) {{
        if (position_ + bytes > data_.size()) {{
            throw ParseError("Unexpected end of data during skip");
        }}
        position_ += bytes;
    }}

    size_t position() const {{ return position_; }}
    size_t remaining() const {{ return data_.size() - position_; }}

private:
    std::span<const uint8_t> data_;
    size_t position_;
}};

class Writer {{
public:
    template<typename T>
    void write_le(T value) {{
        uint8_t bytes[sizeof(T)];
        std::memcpy(bytes, &value, sizeof(T));
        data_.insert(data_.end(), bytes, bytes + sizeof(T));
    }}

    template<typename T>
    void write_be(T value) {{
        for (size_t i = sizeof(T); i > 0; --i) {{
            data_.push_back(static_cast<uint8_t>((value >> ((i - 1) * 8)) & 0xFF));
        }}
    }}

    std::vector<uint8_t> finish() {{ return std::move(data_); }}

private:
    std::vector<uint8_t> data_;
}};

"#,
        namespace
    )
}

pub fn generate_header_end(namespace: &str) -> String {
    format!("\n}} // namespace {}\n", namespace)
}

pub fn generate_struct_declaration(
    struct_name: &str,
    fields: &[(String, String)],
) -> String {
    let mut code = format!("struct {} {{\n", struct_name);

    for (field_name, field_type) in fields {
        code.push_str(&format!("    {} {};\n", field_type, field_name));
    }

    code.push_str(&format!(
        "\n    static {} read(Reader& reader);\n",
        struct_name
    ));
    code.push_str("    void write(Writer& writer) const;\n");
    code.push_str("};\n\n");

    code
}

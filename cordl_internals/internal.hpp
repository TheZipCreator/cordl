#pragma once

#include "config.hpp"
#include <array>
#include <cstddef>
#include <cstring>

namespace cordl_internals {
    namespace internal {
        template <std::size_t sz> struct NTTPString {
            constexpr NTTPString(char const (&n)[sz]) : data{} {
                std::copy_n(n, sz, data.begin());
            }
            std::array<char, sz> data;
        };
    }

    /// @brief gets an offset from a given pointer
    template <std::size_t offset>
    CORDL_HIDDEN constexpr inline void** getAtOffset(void* instance) {
        return reinterpret_cast<void**>(static_cast<uint8_t*>(instance) + offset);
    }

    template <std::size_t sz>
    CORDL_HIDDEN constexpr void copyByByte(std::array<std::byte, sz> const& src, std::array<std::byte, sz>& dst) {
        dst = src;
    }

    template <std::size_t sz>
    CORDL_HIDDEN constexpr void copyByByte(void* src, void* dst) {
        std::memcpy(dst, src, sz);
    }

    template <std::size_t sz>
    CORDL_HIDDEN constexpr void moveByByte(std::array<std::byte, sz>&& src, std::array<std::byte, sz>& dst) {
        dst = std::move(src);
    }

    template <std::size_t sz>
    CORDL_HIDDEN constexpr void moveByByte(void* src, void* dst) {
        std::memmove(dst, src, sz);
    }
}

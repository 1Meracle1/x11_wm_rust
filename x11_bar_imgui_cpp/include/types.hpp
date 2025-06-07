#ifndef TYPES_H
#define TYPES_H

#include <cstdint>
#include <type_traits>

using i8    = std::int8_t;
using i16   = std::int16_t;
using i32   = std::int32_t;
using i64   = std::int64_t;
using u8    = std::uint8_t;
using u16   = std::uint16_t;
using u32   = std::uint32_t;
using u64   = std::uint64_t;
using f32   = float;
using f64   = double;
using isize = i64;
using usize = u64;
using byte  = u8;

using cstring = const char*;
using rawptr  = void*;

#define cast(Type) (Type)

template <typename T>
concept TrivialSmall = std::is_trivially_copyable_v<T> && std::is_standard_layout_v<T> &&
                       std::is_trivially_destructible_v<T> && std::is_trivially_destructible_v<T> && sizeof(T) <= 16;

static_assert(TrivialSmall<u64>);
static_assert(TrivialSmall<cstring>);
static_assert(TrivialSmall<rawptr>);

template <typename T>
concept Scalar = std::is_integral_v<T> || std::is_floating_point_v<T>;

#endif
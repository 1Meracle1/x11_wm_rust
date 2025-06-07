#ifndef SLICE_H
#define SLICE_H

#include "assert.hpp"
#include "types.hpp"
#include <algorithm>
#include <compare>
#include <concepts>
#include <cstring>
#include <functional>
#include <iterator>
#include <type_traits>
#include <iostream>
#include <span>

template <typename T>
concept SliceValueTypeConcept = requires(T t) {
    { !std::is_same_v<T, void> && sizeof(T) != 0 && std::equality_comparable<T> };
};

template <typename F, typename ValueType>
concept SliceElementEqualityPredicate = requires(F f, const ValueType& lhs, const ValueType& rhs) {
    { f(lhs, rhs) } -> std::same_as<bool>;
};

template <typename F, typename ValueType>
concept SliceElementWeakOrderingComparePredicate = requires(F f, const ValueType& lhs, const ValueType& rhs) {
    { f(lhs, rhs) } -> std::same_as<std::weak_ordering>;
};

#define ByteSliceFromCstr(cstr) Slice<const char>{cstr}.chop_zero_termination().reinterpret_elements_as<u8>()

#define ByteSliceFromCstrZeroTerm(cstr) Slice<const char>{cstr}.reinterpret_elements_as<u8>()

template <SliceValueTypeConcept ValueType> struct Slice
{
    using value_type      = ValueType;
    using pointer         = value_type*;
    using const_pointer   = const value_type*;
    using reference       = value_type&;
    using const_reference = std::conditional_t<TrivialSmall<value_type>, value_type, const value_type&>;

    using iterator_category = std::contiguous_iterator_tag;
    using iterator          = pointer;
    using const_iterator    = const_pointer;

    using size_type       = std::size_t;
    using difference_type = std::ptrdiff_t;

    pointer   m_ptr;
    size_type m_len;

  public:
    constexpr Slice() noexcept : m_ptr(nullptr), m_len(0) {}

    constexpr Slice(pointer ptr, size_type len) noexcept : m_ptr(ptr), m_len(len) {}

    // implicit constructor for literal arrays like `int arr[] = {1, 2, 3};`,
    template <size_type N> constexpr explicit Slice(value_type (&arr)[N]) noexcept : m_ptr(arr), m_len(N) {}

    template <typename OtherValueType>
        requires std::is_convertible_v<OtherValueType (*)[], value_type (*)[]>
    constexpr explicit Slice(std::span<OtherValueType> s) noexcept : m_ptr(s.data()), m_len(s.size())
    {
    }

    constexpr explicit operator std::span<value_type>() noexcept { return {m_ptr, m_len}; }
    constexpr explicit operator std::span<const value_type>() const noexcept { return {m_ptr, m_len}; }

    // implicit constructor for intializer lists;
    // SAFETY: rough maniacs governing c++ standards decided it is a good idea,
    // for initializer_list to have internal temp array and have its address
    // returned by the .begin() call.
    // constexpr Slice(std::initializer_list<value_type> init_list) noexcept
    //     : m_ptr(init_list.begin())
    //     , m_len(init_list.size())
    // {
    // }

    // alignment of value_type matches it of NewValueType
    template <SliceValueTypeConcept NewValueType>
    [[nodiscard]] constexpr Slice<NewValueType> reinterpret_elements_as() noexcept
    {
        if (empty())
        {
            return Slice<NewValueType>();
        }
        size_type size_bytes = len() * sizeof(value_type);
        // should we allow data truncation?
        // if (size_bytes % sizeof(NewValueType) != 0) {}
        size_type new_len = size_bytes / sizeof(NewValueType);
        return Slice<NewValueType>(cast(NewValueType*) m_ptr, new_len);
    }

    // [start..)
    [[nodiscard]] constexpr Slice slice_from(size_type start) const noexcept { return slice(start, len()); }

    // [..end)
    [[nodiscard]] constexpr Slice slice_to(size_type end) const noexcept { return slice(0, end); }

    // [end-start..)
    [[nodiscard]] constexpr Slice slice_from_back(size_type length) const noexcept
    {
        return slice(len() - length, len());
    }

    // [start, end)
    [[nodiscard]] constexpr Slice slice(size_type start, size_type end) const noexcept
    {
        Assert(start <= len() && end <= len());
        size_type   len  = end - start;
        value_type* data = len == 0 ? nullptr : m_ptr + start;
        return Slice(data, len);
    }

    [[nodiscard]] constexpr Slice take(size_type count) const { return slice_to(count); }
    [[nodiscard]] constexpr Slice take_max(size_type count) const
    {
        size_type actual_count = std::min(count, len());
        return slice_to(actual_count);
    }

    [[nodiscard]] constexpr Slice drop(size_type count) const { return slice_from(count); }
    [[nodiscard]] constexpr Slice drop_back(size_type count) const
    {
        Assert(count <= len());
        return slice_to(len() - count);
    }

    // [start..)
    [[nodiscard]] constexpr Slice slice_from_unchecked(size_type start) const noexcept
    {
        return slice_unchecked(start, len());
    }

    // [..end)
    [[nodiscard]] constexpr Slice slice_to_unchecked(size_type end) const noexcept { return slice_unchecked(0, end); }

    // [end-start..)
    [[nodiscard]] constexpr Slice slice_from_back_unchecked(size_type length) const noexcept
    {
        return slice_unchecked(len() - length, len());
    }

    // [start, end)
    [[nodiscard]] constexpr Slice slice_unchecked(size_type start, size_type end) const noexcept
    {
        size_type   len  = end - start;
        value_type* data = len == 0 ? nullptr : m_ptr + start;
        return Slice(data, len);
    }

    [[nodiscard]] constexpr pointer       data() noexcept { return m_ptr; }
    [[nodiscard]] constexpr const_pointer data() const noexcept { return m_ptr; }
    [[nodiscard]] constexpr pointer       raw() noexcept { return m_ptr; }
    [[nodiscard]] constexpr const_pointer raw() const noexcept { return m_ptr; }

    [[nodiscard]] constexpr size_type len() const noexcept { return m_len; }
    [[nodiscard]] constexpr bool      empty() const noexcept { return len() == 0; }
    [[nodiscard]] constexpr bool      not_empty() const noexcept { return len() > 0; }

    // clang-format off
    [[nodiscard]] reference       operator[](size_type i) { return m_ptr[i]; }
    [[nodiscard]] const_reference operator[](size_type i) const requires(!TrivialSmall<value_type>) { return m_ptr[i]; }
    [[nodiscard]] value_type      operator[](size_type i) const requires( TrivialSmall<value_type>) { return m_ptr[i]; }

    [[nodiscard]] iterator       begin()        { return m_ptr; }
    [[nodiscard]] iterator       end()          { return m_ptr + len(); }
    [[nodiscard]] const_iterator begin()  const { return m_ptr; }
    [[nodiscard]] const_iterator end()    const { return m_ptr + len(); }
    [[nodiscard]] const_iterator cbegin() const { return m_ptr; }
    [[nodiscard]] const_iterator cend()   const { return m_ptr + len(); }

    [[nodiscard]] std::reverse_iterator<iterator>       rbegin()        { return std::reverse_iterator<iterator>(end()); }
    [[nodiscard]] std::reverse_iterator<iterator>       rend()          { return std::reverse_iterator<iterator>(begin()); }
    [[nodiscard]] std::reverse_iterator<const_iterator> rbegin()  const { return std::reverse_iterator<const_iterator>(end()); }
    [[nodiscard]] std::reverse_iterator<const_iterator> rend()    const { return std::reverse_iterator<const_iterator>(begin()); }
    [[nodiscard]] std::reverse_iterator<const_iterator> crbegin() const { return std::reverse_iterator<const_iterator>(cend()); }
    [[nodiscard]] std::reverse_iterator<const_iterator> crend()   const { return std::reverse_iterator<const_iterator>(cbegin()); }

    [[nodiscard]] reference       front()       { Assert(not_empty()); return m_ptr[0]; }
    [[nodiscard]] const_reference front() const { Assert(not_empty()); return m_ptr[0]; }

    [[nodiscard]] reference       first()       { Assert(not_empty()); return m_ptr[0]; }
    [[nodiscard]] const_reference first() const { Assert(not_empty()); return m_ptr[0]; }

    [[nodiscard]] reference       back()       { Assert(not_empty()); return m_ptr[len() - 1]; }
    [[nodiscard]] const_reference back() const { Assert(not_empty()); return m_ptr[len() - 1]; }

    [[nodiscard]] reference       last()       { Assert(not_empty()); return m_ptr[len() - 1]; }
    [[nodiscard]] const_reference last() const { Assert(not_empty()); return m_ptr[len() - 1]; }
    // clang-format on

    [[nodiscard]] constexpr i64 linear_search(value_type v) const
        requires(Scalar<value_type>)
    {
        for (u64 i = 0; i < m_len; ++i)
        {
            if (m_ptr[i] == v)
                return (i64)i;
        }
        return -1;
    }

    [[nodiscard]] constexpr i64 linear_search(const_reference v) const
    {
        for (size_type i = 0; i < len(); i++)
            if (m_ptr[i] == v)
                return (i64)i;
        return -1;
    }

    using Predicate = std::function<bool(const_reference, const_reference)>;

    [[nodiscard]] constexpr i64 linear_search(const_reference v, Predicate&& predicate) const
    {
        for (size_type i = 0; i < len(); i++)
            if (predicate(m_ptr[i], v))
                return (i64)i;
        return -1;
    }

    [[nodiscard]] constexpr i64 linear_search(Slice<value_type> needle) const
    {
        size_type length     = len();
        size_type needle_len = needle.len();
        if (needle_len == 0 || needle_len > length)
        {
            return -1;
        }
        size_type max_len = length - needle_len + 1;
        for (size_type i = 0; i < max_len; ++i)
        {
            auto sub = slice(i, i + needle_len);
            if (sub.equal(needle))
            {
                return (i64)i;
            }
        }
        return -1;
    }

    [[nodiscard]] constexpr i64 linear_search_any_of(Slice<value_type> s) const
    {
        if (s.empty())
        {
            return -1;
        }
        for (size_type i = 0; i < len(); i++)
        {
            i64 j = s.linear_search(m_ptr[i]);
            if (j != -1)
            {
                return (i64)i;
            }
        }
        return -1;
    }

    [[nodiscard]] constexpr bool contains(const_reference v) const { return linear_search(v) != -1; }

    [[nodiscard]] constexpr bool contains(const_reference v, Predicate&& predicate) const
    {
        return linear_search(v, std::forward<Predicate>(predicate)) != -1;
    }

    [[nodiscard]] constexpr bool contains(Slice<value_type> needle) const { return linear_search(needle) != -1; }

    constexpr void zero()
    {
        if (len() > 0)
            std::memset(m_ptr, 0, len());
    }

    [[nodiscard]] constexpr bool bytes_equal(Slice<value_type> other) const
    {
        if (len() != other.len())
            return false;
        return std::memcmp(m_ptr, other.m_ptr, len() * sizeof(value_type)) == 0;
    }

    [[nodiscard]] constexpr bool equal(Slice<value_type> other) const
        requires(!TrivialSmall<value_type>)
    {
        if (len() != other.len())
            return false;
        for (size_type i = 0; i < len(); ++i)
            if (m_ptr[i] != other[i])
                return false;
        return true;
    }

    [[nodiscard]] constexpr bool equal(Slice<value_type> other) const
        requires(TrivialSmall<value_type>)
    {
        if (other.m_len != m_len)
            return false;
        return std::memcmp(m_ptr, other.m_ptr, m_len * sizeof(value_type)) == 0;
    }

    [[nodiscard]] constexpr bool equal(Slice<value_type> other, Predicate&& predicate) const
    {
        if (len() != other.m_len)
            return false;
        for (size_type i = 0; i < len(); ++i)
            if (!predicate(m_ptr[i], other[i]))
                return false;
        return true;
    }

    friend bool operator==(Slice<value_type> lhs, Slice<value_type> rhs) { return lhs.equal(rhs); }

    [[nodiscard]] constexpr bool starts_with(const_reference v) const
    {
        if (empty())
            return false;
        return m_ptr[0] == v;
    }

    [[nodiscard]] constexpr bool starts_with(Slice<value_type> needle) const
    {
        if (len() < needle.len())
            return false;
        return slice_to(needle.len()).equal(needle);
    }

    [[nodiscard]] constexpr bool starts_with(Slice<value_type> needle, Predicate&& predicate) const
    {
        if (len() < needle.len())
            return false;
        return slice_to(needle.len()).equal(needle, std::forward<Predicate>(predicate));
    }

    [[nodiscard]] constexpr bool ends_with(Slice<value_type> needle) const
    {
        if (len() < needle.len())
            return false;
        return slice_from_back(needle.len()).equal(needle);
    }

    [[nodiscard]] constexpr bool ends_with(Slice<value_type> needle, Predicate&& predicate) const
    {
        if (len() < needle.len())
            return false;
        return slice_from_back(needle.len()).equal(needle, std::forward<Predicate>(predicate));
    }

    void swap(size_type i, size_type j) { std::swap(m_ptr[i], m_ptr[j]); }

    void reverse()
    {
        size_type half = len() / 2;
        for (size_type i = 0; i < half; i++)
            swap(i, len() - i - 1);
    }

    void unique()
    {
        if (len() < 2)
            return *this;
        size_type i = 1;
        for (size_type j = 1; j < len(); j++)
        {
            if (m_ptr[j] != m_ptr[j - 1])
            {
                if (i != j)
                    m_ptr[i] = m_ptr[j];
                i += 1;
            }
        }
        return slice_to(i);
    }

    template <SliceElementEqualityPredicate<value_type> F> constexpr Slice unique(F predicate)
    {
        if (len() < 2)
            return *this;
        size_type i = 1;
        for (size_type j = 1; j < len(); j++)
        {
            if (!predicate(m_ptr[j], m_ptr[j - 1]))
            {
                if (i != j)
                    m_ptr[i] = m_ptr[j];
                i += 1;
            }
        }
        return slice_to(i);
    }

    template <typename F> constexpr ValueType reduce(const_reference initial_value, F&& f) const
    {
        auto res = initial_value;
        for (size_type j = 0; j < len(); j++)
            res = f(res, m_ptr[j]);
        return res;
    }

    template <SliceElementWeakOrderingComparePredicate<value_type> F> constexpr void sort(F&& f)
    {
        std::sort(begin(), end(), std::forward<SliceElementWeakOrderingComparePredicate<F, value_type>>(f));
    }

    constexpr void split_once(const_reference sep, Slice& lhs, Slice& rhs) const
    {
        i64 index = linear_search(sep);
        if (index == -1)
        {
            lhs.m_len = m_len;
            lhs.m_ptr = m_ptr;
            rhs.m_len = 0;
            rhs.m_ptr = nullptr;
        }
        else
        {
            lhs.m_len = index;
            lhs.m_ptr = m_ptr;
            rhs.m_len = m_len - index - 1;
            rhs.m_ptr = m_ptr + index + 1;
        }
    }

    constexpr void split_at(size_type mid, Slice& lhs, Slice& rhs) const
    {
        Assert(mid >= 0 && mid <= len());
        lhs = slice_to(mid);
        rhs = slice_from(mid);
    }

    // SAFETY: Caller has to check that `0 <= mid <= self.len()`
    constexpr void split_at_unchecked(size_type mid, Slice& lhs, Slice& rhs) const
    {
        lhs = slice_to_unchecked(mid);
        rhs = slice_from_unchecked(mid);
    }

    [[nodiscard]] constexpr Slice<value_type> until(const_reference v) const
    {
        i64 i = linear_search(v);
        return i == -1 ? *this : slice_to(i);
    }

    [[nodiscard]] constexpr Slice<value_type> trim_left_not_equal(const_reference v) const
    {
        i64 i = linear_search(v);
        return i == -1 ? *this : slice_from(i);
        return slice_from(i);
    }

    [[nodiscard]] constexpr Slice<value_type> trim_left(Slice<value_type> trimmed_elements) const
    {
        size_type i = 0;
        for (; i < len(); i++)
        {
            if (trimmed_elements.linear_search(m_ptr[i]) == -1)
            {
                break;
            }
        }
        return slice_from(i);
    }

    [[nodiscard]] constexpr Slice<value_type> trim_left(const_reference v) const
    {
        size_type i = 0;
        for (; i < len(); i++)
        {
            if (m_ptr[i] != v)
            {
                break;
            }
        }
        return slice_from(i);
    }

    [[nodiscard]] constexpr Slice<value_type> trim_right_not_equal(const_reference v) const
    {
        i64 i = m_len - 1;
        for (; i >= 0; i--)
        {
            if (m_ptr[i] == v)
            {
                break;
            }
        }
        return slice_to(i);
    }

    [[nodiscard]] constexpr Slice<value_type> trim_right(Slice<value_type> trimmed_elements) const
    {
        i64 i = m_len - 1;
        for (; i >= 0; i--)
        {
            if (trimmed_elements.linear_search(m_ptr[i]) == -1)
            {
                break;
            }
        }
        return slice_to(i);
    }

    [[nodiscard]] constexpr Slice<value_type> trim_right(const_reference v) const
    {
        i64 i = m_len - 1;
        for (; i >= 0; i--)
        {
            if (m_ptr[i] != v)
            {
                break;
            }
        }
        return slice_to(i);
    }

    [[nodiscard]] constexpr Slice<value_type> trim(Slice<value_type> trimmed_elements) const
    {
        return trim_left(trimmed_elements).trim_right(trimmed_elements);
    }

    [[nodiscard]] Slice<value_type> trim_spaces() const
    {
        static auto SPACE_CHARS = Slice<const char>(" \t\n\r").reinterpret_elements_as<u8>();
        return trim_left(SPACE_CHARS).trim_right(SPACE_CHARS);
    }

    [[nodiscard]] constexpr Slice chop_zero_termination() const
    {
        if (m_len > 0 && m_ptr[m_len - 1] == 0)
        {
            return slice_to(m_len - 1);
        }
        return *this;
    }
};

inline std::ostream& operator<<(std::ostream& os, Slice<const char> str)
{
    if (str.len() > 0)
    {
        os.write(cast(const char*)(str.begin()), (std::streamsize)str.len());
    }
    return os;
}

inline std::ostream& operator<<(std::ostream& os, Slice<u8> str)
{
    if (str.len() > 0)
    {
        os.write(cast(const char*)(str.begin()), (std::streamsize)str.len());
    }
    return os;
}

namespace std
{
template <> struct hash<Slice<u8>>
{
    size_t operator()(Slice<u8> s) const noexcept
    {
        size_t seed = 0;
        for (uint64_t i = 0; i < s.len(); ++i)
        {
            seed ^= std::hash<u8>{}(s.m_ptr[i]) + 0x9e3779b9 + (seed << 6) + (seed >> 2);
        }
        return seed;
    }
};
} // namespace std

#endif
#ifndef PARSE_NUM_H
#define PARSE_NUM_H

#include "slice.hpp"
#include <cmath>
#include <concepts>
#include <limits>
#include <ostream>

enum class ParseIntFromStringError
{
    None,
    EmptyString,
    NoDigitsFound,
    UnexpectedNegativeSign,
    UnexpectedNonNumberCharacter,
    Overflow,
};

inline std::ostream& operator<<(std::ostream& os, ParseIntFromStringError e)
{
    switch (e)
    {
    case ParseIntFromStringError::None:
        os << "ParseIntFromStringError::None";
        break;
    case ParseIntFromStringError::EmptyString:
        os << "ParseIntFromStringError::EmptyString";
        break;
    case ParseIntFromStringError::NoDigitsFound:
        os << "ParseIntFromStringError::NoDigitsFound";
        break;
    case ParseIntFromStringError::UnexpectedNegativeSign:
        os << "ParseIntFromStringError::UnexpectedNegativeSign";
        break;
    case ParseIntFromStringError::UnexpectedNonNumberCharacter:
        os << "ParseIntFromStringError::UnexpectedNonNumberCharacter";
        break;
    case ParseIntFromStringError::Overflow:
        os << "ParseIntFromStringError::Overflow";
        break;
    }
    return os;
}

template <std::integral T> static ParseIntFromStringError parse_int(Slice<u8> str, T& number, u8 ignore_c = ',')
{
    if (str.empty())
    {
        return ParseIntFromStringError::EmptyString;
    }
    i64  sign  = 1;
    auto it    = str.begin();
    auto itEnd = str.end();
    if (u8 first = *it; first == '+')
    {
        it++;
    }
    else if (first == '-')
    {
        if constexpr (std::signed_integral<T>)
        {
            it++;
            sign *= -1;
        }
        else
        {
            return ParseIntFromStringError::UnexpectedNegativeSign;
        }
    }

    if (it == itEnd)
    {
        return ParseIntFromStringError::NoDigitsFound;
    }

    constexpr T MAX_VALUE = std::numeric_limits<T>::max();
    constexpr T MIN_VALUE = std::numeric_limits<T>::min();

    T result = 0;
    for (; it != itEnd; ++it)
    {
        if (*it == ignore_c)
            continue;

        if (*it < '0' || *it > '9')
        {
            return ParseIntFromStringError::UnexpectedNonNumberCharacter;
        }
        u8 digit_value = *it - '0';
        if (sign == -1)
        {
            if constexpr (std::signed_integral<T>)
            {
                if (result < MIN_VALUE / 10 || (result == MIN_VALUE / 10 && cast(T)(-digit_value) < MIN_VALUE % 10))
                {
                    return ParseIntFromStringError::Overflow;
                }
                result = result * 10 - digit_value;
            }
        }
        else
        {
            if (result > MAX_VALUE / 10 || (result == MAX_VALUE / 10 && cast(T) digit_value > MAX_VALUE % 10))
            {
                return ParseIntFromStringError::Overflow;
            }
            result = result * 10 + digit_value;
        }
    }
    number = result;
    return ParseIntFromStringError::None;
}

enum class ParseFloatFromStringError
{
    Ok,
    EmptyString,
    NoDigitsFound,
    NoFractionalPartDigits,
    UnexpectedNonNumberCharacter,
    Overflow,
};

/*
    MeasureTimeMicro("parsing of negative float without fractional part");
    Slice<const char> cstr   = "-1";
    Slice<u8>         str    = cstr.chop_zero_termination().reinterpret_elements_as<u8>();
    f64               number = 0;
    Assert(parse_float(str, number, ',') == ParseFloatFromStringError::None);
    Assert(number == cast(f64)(-1.0));
*/
template <std::floating_point T>
[[nodiscard]] static ParseFloatFromStringError parse_float(Slice<u8> str, T& number, u8 separator = '.')
{
    if (str.empty())
    {
        return ParseFloatFromStringError::EmptyString;
    }

    i64 whole      = 0;
    u64 fractional = 0;

    i64 sep_idx = str.linear_search(separator);
    if (sep_idx + 1 >= cast(i64) str.len())
    {
        return ParseFloatFromStringError::NoFractionalPartDigits;
    }

    Slice<u8> whole_part_str = str;
    if (sep_idx != -1)
    {
        whole_part_str = str.slice_to((u64)sep_idx);
    }

    auto part_parse_error = parse_int(whole_part_str, whole);
    switch (part_parse_error)
    {
    case ParseIntFromStringError::EmptyString:
        return ParseFloatFromStringError::EmptyString;
    case ParseIntFromStringError::NoDigitsFound:
        return ParseFloatFromStringError::NoDigitsFound;
    case ParseIntFromStringError::UnexpectedNegativeSign:
    case ParseIntFromStringError::UnexpectedNonNumberCharacter:
        return ParseFloatFromStringError::UnexpectedNonNumberCharacter;
    case ParseIntFromStringError::Overflow:
        return ParseFloatFromStringError::Overflow;
        break;
    default:
        break;
    }

    if (sep_idx != -1)
    {
        Slice<u8> fract_part_str = str.slice_from((u64)(sep_idx + 1));
        part_parse_error         = parse_int(fract_part_str, whole);
        switch (part_parse_error)
        {
        case ParseIntFromStringError::EmptyString:
            return ParseFloatFromStringError::EmptyString;
        case ParseIntFromStringError::NoDigitsFound:
            return ParseFloatFromStringError::NoDigitsFound;
        case ParseIntFromStringError::UnexpectedNegativeSign:
        case ParseIntFromStringError::UnexpectedNonNumberCharacter:
            return ParseFloatFromStringError::UnexpectedNonNumberCharacter;
        case ParseIntFromStringError::Overflow:
            return ParseFloatFromStringError::Overflow;
            break;
        default:
            break;
        }

        T divisor          = cast(T) std::pow(cast(f64) 10.0, cast(f64) fract_part_str.len());
        T fractional_value = cast(T) fractional / divisor;
        number             = cast(T) whole + fractional_value;
    }
    else
    {
        number = cast(T) whole;
    }

    return ParseFloatFromStringError::Ok;
}

#endif
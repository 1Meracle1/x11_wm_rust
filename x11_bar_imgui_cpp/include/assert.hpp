#ifndef ASSERT_H
#define ASSERT_H

#include <cstdio>
#include <cstdlib>
#include <format>
#include <source_location>

#define Assert(cond)                                                                                                   \
    if (!(cond))                                                                                                       \
    assert_impl(#cond)
#define Assertr(cond, reason)                                                                                          \
    if (!(cond))                                                                                                       \
    assert_impl(#cond, (reason))
#define Assertv(cond, value)                                                                                           \
    if (!(cond))                                                                                                       \
    assert_impl(#cond, std::format("{}", value))

// clang-format off
#if defined(_WIN32)
    #define WIN32_LEAN_AND_MEAN
    #define NOMINMAX
    #include <windows.h>
    #define DEBUG_BREAK() __debugbreak()
    inline bool is_debugger_attached() { return IsDebuggerPresent(); }
#elif defined(__unix__) || defined(__APPLE__)
    #include <csignal>
    #include <sys/types.h>
    #include <unistd.h>
    #if defined(__linux__)
    #include <sys/ptrace.h>
    #endif

    #define DEBUG_BREAK() raise(SIGTRAP)

    inline bool is_debugger_attached()
    {
    #if defined(__linux__)
        // On Linux, check if being traced
        return ptrace(PTRACE_TRACEME, 0, 1, 0) == -1;
    #elif defined(__APPLE__)
        // TODO: On macOS, use sysctl
        return false; // Implement as needed
    #else
        return false;
    #endif
    }
#else
    #define DEBUG_BREAK() ((void)0)
    inline bool is_debugger_attached() { return false; }
#endif
// clang-format on

static void assert_impl(std::string_view     condition,
                        std::string_view     value           = "",
                        std::string_view     message         = "",
                        std::source_location source_location = std::source_location::current())
{
    // clang-format off
    std::printf(
        "%s\n",
        std::format(
            "Assertion failed: ( {} ){} at ( {}:{} ) in function ( {} ){}", 
            condition,
            value != "" ? std::format(" actual value ( {} )", value) : value, 
            source_location.file_name(), source_location.line(), source_location.function_name(),
            message != "" ? std::format(" with error message: ( {} )", message) : ""
        ).c_str()
    );
    // clang-format on
    if (is_debugger_attached())
        DEBUG_BREAK();
    std::abort();
}

#endif
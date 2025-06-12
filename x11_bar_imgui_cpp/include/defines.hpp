#ifndef DEFINES_H
#define DEFINES_H

#if defined(__linux__)
    #define OS_LINUX 1
#elif defined(_WIN32)
    #define OS_WIN32 1
    #define OS_WINDOWS 1
#elif defined(__APPLE__)
    #define OS_APPLE 1
    #define OS_DARWIN 1
    #define OS_MACOS 1
#elif defined(__unix__)
    #define OS_UNIX 1
#endif

#define Kilobytes(x) (         (x) * (u64)(1024))
#define Megabytes(x) (Kilobytes(x) * (u64)(1024))
#define Gigabytes(x) (Megabytes(x) * (u64)(1024))
#define Terabytes(x) (Gigabytes(x) * (u64)(1024))

#define CONCATENATE_DETAIL(x, y) x##y
#define CONCATENATE(x, y) CONCATENATE_DETAIL(x, y)

#endif
#include "unix_socket.hpp"
#include "assert.hpp"
#include <cerrno>
#include <cstddef>
#include <filesystem>
#include <sys/ioctl.h>
#include <unistd.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <utility>

UnixStream UnixStream::connect(std::string_view path)
{
    if (!std::filesystem::exists(path))
    {
        return UnixStream(-1);
    }
    int socket_fd = socket(AF_UNIX, SOCK_STREAM, 0);

    sockaddr_un sock_addr{};
    sock_addr.sun_family = AF_UNIX;
    strncpy(sock_addr.sun_path, path.data(), path.size());
    std::size_t sock_len = offsetof(sockaddr_un, sun_path) + path.size() + 1;

    int result_connect = ::connect(socket_fd, (const sockaddr*)(&sock_addr), (socklen_t)sock_len);
    if (result_connect == -1)
    {
        return UnixStream(-1);
    }
    return UnixStream(socket_fd);
}

[[nodiscard]] bool UnixStream::is_ok() const { return m_socket_fd != -1; }

UnixStream::UnixStream(UnixStream&& other) : m_socket_fd(std::exchange(other.m_socket_fd, -1)) {}
UnixStream& UnixStream::operator=(UnixStream&& other)
{
    m_socket_fd = std::exchange(other.m_socket_fd, -1);
    return *this;
}
UnixStream::~UnixStream()
{
    if (m_socket_fd != -1)
    {
        close(m_socket_fd);
        m_socket_fd = -1;
    }
}

[[nodiscard]] UnixError UnixStream::read_exact(std::size_t bytes_len, std::vector<char>& bytes) const
{
    if (!is_ok())
    {
        return UnixError::InvalidInstance;
    }
    Assert(bytes_len > 0);
    bytes.clear();
    bytes.resize(bytes_len);
    char*       buffer     = bytes.data();
    std::size_t total_read = 0;
    while (total_read < bytes_len)
    {
        ssize_t bytes_read = ::read(m_socket_fd, buffer + total_read, bytes_len - total_read);
        // std::cout << "read from socket: " << m_socket_fd << ", bytes_read: " << bytes_read << ", out of: " <<
        // bytes_len
        //           << '\n';
        if (bytes_read == -1)
        {
            if (errno == EINTR)
            {
                continue;
            }
            ::perror("got error trying to read from socket");
            return UnixError::CommunicationError;
        }
        if (bytes_read == 0)
        {
            return UnixError::Eof;
        }
        total_read += (size_t)bytes_read;
    }
    // std::cout << "completed read from socket: " << m_socket_fd << ", total_read: " << total_read
    //           << ", out of: " << bytes_len << '\n';
    return UnixError::Ok;
}

[[nodiscard]] UnixError UnixStream::write_all(const std::vector<char>& bytes) const
{
    if (!is_ok())
    {
        return UnixError::InvalidInstance;
    }
    std::size_t total_to_write = bytes.size();
    std::size_t total_written  = 0;
    const char* buffer         = bytes.data();
    while (total_written < total_to_write)
    {
        ssize_t bytes_written = ::write(m_socket_fd, buffer + total_written, total_to_write - total_written);
        if (bytes_written == -1)
        {
            if (errno == EINTR)
            {
                continue;
            }
            ::perror("got error trying to write to socket");
            return UnixError::CommunicationError;
        }
        total_written += (std::size_t)bytes_written;
    }
    return UnixError::Ok;
}

[[nodiscard]] bool UnixStream::set_nonblocking(bool non_blocking)
{
    if (!is_ok())
    {
        return false;
    }
    int non_blocking_raw = (int)non_blocking;
    int result           = ::ioctl(m_socket_fd, FIONBIO, &non_blocking_raw);
    if (result == -1)
    {
        close(m_socket_fd);
        m_socket_fd = -1;
    }
    return true;
}

UnixStream::UnixStream(int socket_fd) : m_socket_fd(socket_fd) {}

// ----------------------------------------------------------------------

UnixListener UnixListener::bind(std::string_view path)
{
    ::unlink(path.data());
    int socket_fd = socket(AF_UNIX, SOCK_STREAM, 0);

    sockaddr_un sock_addr{};
    sock_addr.sun_family = AF_UNIX;
    strncpy(sock_addr.sun_path, path.data(), path.size());
    std::size_t sock_len = offsetof(sockaddr_un, sun_path) + path.size() + 1;

    int bind_result = ::bind(socket_fd, (const sockaddr*)(&sock_addr), (socklen_t)sock_len);
    if (bind_result == -1)
    {
        perror("failed to bind unix listener\n");
        return UnixListener(-1);
    }

    int listen_result = ::listen(socket_fd, SOMAXCONN);
    if (listen_result == -1)
    {
        perror("failed to start listening on unix listener\n");
        return UnixListener(-1);
    }

    return UnixListener(socket_fd);
}

[[nodiscard]] UnixStream UnixListener::accept() const
{
    if (!is_ok())
    {
        return UnixStream(-1);
    }
    // sockaddr_un sock_addr{};
    // auto        sock_len         = (socklen_t)sizeof(sock_addr);
    int accept_socket_fd = ::accept(m_socket_fd, nullptr, nullptr);
    if (accept_socket_fd == -1)
    {
        perror("failed to accept new socket on unix listener\n");
        return UnixStream(-1);
    }
    return UnixStream(accept_socket_fd);
}

[[nodiscard]] bool UnixListener::is_ok() const { return m_socket_fd != -1; }

[[nodiscard]] bool UnixListener::set_nonblocking(bool non_blocking)
{
    if (!is_ok())
    {
        return false;
    }
    int non_blocking_raw = (int)non_blocking;
    int result           = ::ioctl(m_socket_fd, FIONBIO, &non_blocking_raw);
    if (result == -1)
    {
        close(m_socket_fd);
        m_socket_fd = -1;
    }
    return true;
}

UnixListener::UnixListener(UnixListener&& other) : m_socket_fd(std::exchange(other.m_socket_fd, -1)) {}
UnixListener& UnixListener::operator=(UnixListener&& other)
{
    m_socket_fd = std::exchange(other.m_socket_fd, -1);
    return *this;
}
UnixListener::~UnixListener()
{
    if (m_socket_fd != -1)
    {
        close(m_socket_fd);
        m_socket_fd = -1;
    }
}

UnixListener::UnixListener(int socket_fd) : m_socket_fd(socket_fd) {}

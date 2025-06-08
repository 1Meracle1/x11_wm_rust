#ifndef UNIX_SOCKET_H
#define UNIX_SOCKET_H

#include <condition_variable>
#include <mutex>
#include <string_view>
#include <vector>

class UnixListener;

class UnixStream
{
  public:
    static UnixStream connect(std::string_view path);

    [[nodiscard]] bool is_ok() const;

    UnixStream(const UnixStream&)            = delete;
    UnixStream& operator=(const UnixStream&) = delete;
    UnixStream(UnixStream&& other);
    UnixStream& operator=(UnixStream&& other);
    ~UnixStream();

    [[nodiscard]] bool read_exact(std::size_t bytes_len, std::vector<char>& bytes) const;

    [[nodiscard]] bool write_all(const std::vector<char>& bytes) const;

    [[nodiscard]] bool set_nonblocking(bool non_blocking);

    [[nodiscard]] int socket_fd() const { return m_socket_fd; }

  private:
    explicit UnixStream(int socket_fd);

  private:
    int                     m_socket_fd = -1;
    std::mutex              m_mutex{};
    std::condition_variable m_cv{};

    friend UnixListener;
};

class UnixListener
{
  public:
    static UnixListener bind(std::string_view path);

    [[nodiscard]] bool is_ok() const;

    UnixListener(const UnixListener&)            = delete;
    UnixListener& operator=(const UnixListener&) = delete;
    UnixListener(UnixListener&& other);
    UnixListener& operator=(UnixListener&& other);
    ~UnixListener();

    [[nodiscard]] UnixStream accept() const;

    [[nodiscard]] bool set_nonblocking(bool non_blocking);

    [[nodiscard]] int socket_fd() const { return m_socket_fd; }

  private:
    explicit UnixListener(int socket_fd);

  private:
    int m_socket_fd = -1;
};

#endif
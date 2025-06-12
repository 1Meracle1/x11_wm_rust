#ifndef UNIX_COMMMUNICATION_BUS_H
#define UNIX_COMMMUNICATION_BUS_H

#include "unix_socket.hpp"
#include <mutex>
#include <tbb/concurrent_queue.h>
#include <tbb/concurrent_hash_map.h>
#include <sys/epoll.h>
#include <iostream>
#include <thread>

class UnixCommunicationBus
{
  public:
    explicit UnixCommunicationBus(const char* unix_socket_path)
        : m_unix_stream(UnixStream::connect(unix_socket_path)),
          m_thread([this](const std::stop_token& token) { this->listen_input_messages(token); })
    {
    }

    ~UnixCommunicationBus()
    {
        if (m_epoll_fd != -1)
        {
            close(m_epoll_fd);
            m_epoll_fd = -1;
        }
    }

    UnixCommunicationBus(const UnixCommunicationBus&)            = delete;
    UnixCommunicationBus& operator=(const UnixCommunicationBus&) = delete;
    UnixCommunicationBus(UnixCommunicationBus&&)                 = delete;
    UnixCommunicationBus& operator=(UnixCommunicationBus&&)      = delete;

    // expected for the `msg` to already contain size in the first 8 bytes - sizeof(std::size_t)
    void notify_server(std::vector<char>&& msg)
    {
        if (m_unix_stream.is_ok())
        {
            std::lock_guard<std::mutex> lock_guard(m_mutex);
            if (m_unix_stream.write_all(std::forward<std::vector<char>>(msg)) != UnixError::Ok)
            {
                perror("failed to write message to unix server");
            }
        }
        else
        {
            std::cerr << "failed to notify unix server as the unix stream is not opened\n";
        }
    }

    [[nodiscard]] bool try_pop_input_message(std::vector<char>& res) { return m_input_msgs_queue.try_pop(res); }

  private:
    void listen_input_messages(const std::stop_token& token)
    {
        if (!m_unix_stream.is_ok())
        {
            return;
            std::cerr << "failed to create unix stream\n";
        }
        m_epoll_fd = ::epoll_create1(0);
        if (m_epoll_fd == -1)
        {
            perror("failed to create epoll instance\n");
            return;
        }

        epoll_event event{};
        event.events      = EPOLLIN;
        event.data.fd     = m_unix_stream.socket_fd();
        int epoll_add_res = ::epoll_ctl(m_epoll_fd, EPOLL_CTL_ADD, m_unix_stream.socket_fd(), &event);
        if (epoll_add_res == -1)
        {
            close(m_epoll_fd);
            m_epoll_fd = -1;
            perror("failed to add unix stream socket to epoll\n");
            return;
        }

        std::vector<char>    len_bytes{};
        std::vector<char>    msg_bytes{};
        constexpr static int MAX_EVENTS = 10;
        epoll_event          polled_events[MAX_EVENTS];

        while (!token.stop_requested() && m_epoll_fd != -1)
        {
            int num_new_events = ::epoll_wait(m_epoll_fd, polled_events, MAX_EVENTS, -1);
            for (int i = 0; i < num_new_events; ++i)
            {
                int socket_fd = polled_events[i].data.fd;
                if (socket_fd == m_unix_stream.socket_fd())
                {
                    std::lock_guard<std::mutex> lock_guard(m_mutex);
                    if (auto read_size_result = m_unix_stream.read_exact(sizeof(std::size_t), len_bytes);
                        read_size_result == UnixError::Ok)
                    {
                        std::size_t msg_len = *reinterpret_cast<std::size_t*>(len_bytes.data());
                        if (auto read_msg_result = m_unix_stream.read_exact(msg_len, msg_bytes);
                            read_msg_result == UnixError::Ok)
                        {
                            m_input_msgs_queue.push(std::move(msg_bytes));
                        }
                        else
                        {
                            std::cerr << "failed to read message of size: " << msg_len
                                      << ", result: " << read_msg_result << '\n';
                        }
                    }
                    else
                    {
                        std::cerr << "failed to read size of the message: " << read_size_result << '\n';
                    }
                }
            }
        }
        std::cout << "stopping event_loop_clients_to_server\n";

        close(m_epoll_fd);
        m_epoll_fd = -1;
    }

  private:
    int                                              m_epoll_fd = -1;
    tbb::concurrent_bounded_queue<std::vector<char>> m_input_msgs_queue{};
    std::mutex                                       m_mutex;
    UnixStream                                       m_unix_stream;
    std::jthread                                     m_thread;
};

#endif
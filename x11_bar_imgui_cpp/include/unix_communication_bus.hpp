#ifndef UNIX_COMMMUNICATION_BUS_H
#define UNIX_COMMMUNICATION_BUS_H

#include "unix_socket.hpp"
#include <memory>
#include <mutex>
#include <tbb/concurrent_queue.h>
#include <tbb/concurrent_hash_map.h>
#include <sys/epoll.h>
#include <iostream>
#include <algorithm>
#include <list>
#include <thread>

class UnixCommunicationBus
{
  public:
    explicit UnixCommunicationBus(const char* unix_socket_path)
        : m_unix_listener(UnixListener::bind(unix_socket_path)),
          m_clients_to_server_thread([this](const std::stop_token& token)
                                     { this->event_loop_clients_to_server(token); }),
          m_server_to_clients_thread([this](const std::stop_token& token)
                                     { this->event_loop_server_to_clients(token); })
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

    // expected for the `msg` to already contain size in the first 8 bytes - sizeof(std::size_t)
    void notify_all_clients(const std::vector<char>& msg) { m_server_to_clients_msgs_queue.push(msg); }
    void notify_all_clients(std::vector<char>&& msg)
    {
        m_server_to_clients_msgs_queue.push(std::forward<std::vector<char>>(msg));
    }

    [[nodiscard]] bool try_pop_client_message(std::vector<char>& res)
    {
        return m_clients_to_server_msgs_queue.try_pop(res);
    }

  private:
    void event_loop_clients_to_server(const std::stop_token& token)
    {
        if (!m_unix_listener.is_ok())
        {
            return;
            std::cerr << "failed to create unix listener\n";
        }
        m_epoll_fd = ::epoll_create1(0);
        if (m_epoll_fd == -1)
        {
            // std::cerr << "failed to create epoll instance\n";
            perror("failed to create epoll instance\n");
            return;
        }

        epoll_event event{};
        event.events      = EPOLLIN;
        event.data.fd     = m_unix_listener.socket_fd();
        int epoll_add_res = ::epoll_ctl(m_epoll_fd, EPOLL_CTL_ADD, m_unix_listener.socket_fd(), &event);
        if (epoll_add_res == -1)
        {
            close(m_epoll_fd);
            m_epoll_fd = -1;
            // std::cerr << "failed to add unix listener socket to epoll\n";
            perror("failed to add unix listener socket to epoll\n");
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
                if (socket_fd == m_unix_listener.socket_fd())
                {
                    auto new_unix_stream = m_unix_listener.accept();
                    if (new_unix_stream.is_ok())
                    {
                        int new_client_socket_fd = new_unix_stream.socket_fd();
                        std::cout << "new client socket recieved: " << new_client_socket_fd << '\n';
                        event         = {};
                        event.events  = EPOLLIN;
                        event.data.fd = new_client_socket_fd;
                        epoll_add_res = ::epoll_ctl(m_epoll_fd, EPOLL_CTL_ADD, new_client_socket_fd, &event);
                        if (epoll_add_res == -1)
                        {
                            std::cerr << "failed to add new unix client socket to epoll, fd: " << new_client_socket_fd
                                      << ", number of existing clients: " << m_unix_clients.size() << '\n';
                            continue;
                        }
                        m_unix_clients.emplace_back(std::move(new_unix_stream));
                    }
                }
                else
                {
                    auto it = std::find_if(m_unix_clients.begin(), m_unix_clients.end(),
                                           [searched_socket_fd = socket_fd](const auto& client)
                                           { return searched_socket_fd == client.socket_fd(); });
                    if (it != m_unix_clients.end())
                    {
                        enqueue_unix_client_fd(socket_fd);
                        {
                            // std::cout << "reading length of the new message\n";
                            if (it->read_exact(sizeof(std::size_t), len_bytes))
                            {
                                std::size_t msg_len = *reinterpret_cast<std::size_t*>(len_bytes.data());
                                // std::cout << "reading message body of length: " << msg_len << '\n';
                                if (it->read_exact(msg_len, msg_bytes))
                                {
                                    m_clients_to_server_msgs_queue.push(std::move(msg_bytes));
                                }
                                else
                                {
                                    std::cerr << "failed to read message of size: " << msg_len << '\n';
                                }
                            }
                            else
                            {
                                std::cerr << "failed to read size of the message\n";
                            }
                        }
                        dequeue_unix_client_fd(socket_fd);
                    }
                    else
                    {
                        std::cerr << "no unix client found for the given socket fd: " << socket_fd << '\n';
                    }
                }
            }
        }
        std::cout << "stopping event_loop_clients_to_server\n";

        close(m_epoll_fd);
        m_epoll_fd = -1;
    }

    void event_loop_server_to_clients(const std::stop_token& token)
    {
        std::vector<char> msg{};
        while (!token.stop_requested())
        {
            msg.clear();
            m_server_to_clients_msgs_queue.pop(msg);
            for (auto& client : m_unix_clients)
            {
                int socket_fd = client.socket_fd();

                enqueue_unix_client_fd(socket_fd);
                if (!client.write_all(msg))
                {
                    std::cerr << "failed to write message to client: " << socket_fd << '\n';
                }
                enqueue_unix_client_fd(socket_fd);
            }
        }
    }

  private:
    struct SyncPayload
    {
        std::mutex              mtx;
        std::condition_variable cv;
        bool                    is_being_removed = false;
    };
    using concurrent_set_t = tbb::concurrent_hash_map<int, std::shared_ptr<SyncPayload>>;

    void enqueue_unix_client_fd(int client_fd)
    {
        while (true)
        {
            concurrent_set_t::accessor acc;
            auto                       new_payload = std::make_shared<SyncPayload>();
            if (m_locked_unix_client_fds.insert(acc, client_fd))
            {
                acc->second = new_payload;
                return;
            }
            std::shared_ptr<SyncPayload> existing_payload = acc->second;
            acc.release();
            {
                std::unique_lock<std::mutex> lock(existing_payload->mtx);
                existing_payload->cv.wait(lock, [&] { return existing_payload->is_being_removed; });
            }
        }
    }

    void dequeue_unix_client_fd(int client_fd)
    {
        concurrent_set_t::const_accessor acc;
        if (!m_locked_unix_client_fds.find(acc, client_fd))
        {
            return;
        }
        {
            std::unique_lock<std::mutex> lock(acc->second->mtx);
            acc->second->is_being_removed = true;
        }
        acc->second->cv.notify_all();
        m_locked_unix_client_fds.erase(acc);
    }

  private:
    UnixListener                                     m_unix_listener;
    int                                              m_epoll_fd = -1;
    std::list<UnixStream>                            m_unix_clients{};
    concurrent_set_t                                 m_locked_unix_client_fds{};
    tbb::concurrent_bounded_queue<std::vector<char>> m_server_to_clients_msgs_queue{};
    tbb::concurrent_bounded_queue<std::vector<char>> m_clients_to_server_msgs_queue{};
    std::jthread                                     m_clients_to_server_thread;
    std::jthread                                     m_server_to_clients_thread;
};

#endif
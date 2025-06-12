#ifndef MESSAGE_H
#define MESSAGE_H

#include "assert.hpp"
#include <cstdint>
#include <cstring>
#include <iostream>
#include <optional>
#include <sys/types.h>
#include <type_traits>
#include <vector>

enum class MessageType : uint8_t
{
    KeyboardLayout    = 0,
    WorkspaceList     = 1,
    WorkspaceActive   = 2,
    RequestClientInit = 3,
};

struct RequestClientInit
{
};

union MessageRaw
{
    std::string                keyboard_layout_name;
    std::uint32_t              workspace_active_id = 0;
    std::vector<std::uint32_t> workspace_list_ids;
    RequestClientInit          request_client_init;

    MessageRaw() {}
    ~MessageRaw() {}
};

template <MessageType E> struct MessageTypeDataType;

template <> struct MessageTypeDataType<MessageType::KeyboardLayout>
{
    using type = std::string;
};

template <> struct MessageTypeDataType<MessageType::WorkspaceList>
{
    using type = std::vector<uint32_t>;
};

template <> struct MessageTypeDataType<MessageType::WorkspaceActive>
{
    using type = std::uint32_t;
};

template <> struct MessageTypeDataType<MessageType::RequestClientInit>
{
    using type = RequestClientInit;
};

template <MessageType E> using MessageTypeDataTypeT = typename MessageTypeDataType<E>::type;

template <auto E> inline constexpr std::integral_constant<decltype(E), E> tag{};

class Message
{
  public:
    template <MessageType E, typename T>
    Message(std::integral_constant<MessageType, E> type_tag, T&& value)
        requires std::is_same_v<MessageTypeDataTypeT<E>, std::decay_t<T>>
        : m_message_type(E), m_destroyed(false)
    {
        (void)type_tag;
        if constexpr (E == MessageType::KeyboardLayout)
        {
            new (&m_message_raw.keyboard_layout_name) std::string(std::forward<T>(value));
        }
        else if constexpr (E == MessageType::WorkspaceActive)
        {
            new (&m_message_raw.workspace_active_id) uint32_t(std::forward<T>(value));
        }
        else if constexpr (E == MessageType::WorkspaceList)
        {
            new (&m_message_raw.workspace_list_ids) std::vector(std::forward<T>(value));
        }
    }

    ~Message() { deinit(); }

    Message(const Message&)                = delete;
    Message& operator=(const MessageType&) = delete;

    Message(Message&& other) : m_message_type(other.m_message_type), m_destroyed(other.m_destroyed)
    {
        if (!m_destroyed)
        {
            switch (m_message_type)
            {
            case MessageType::KeyboardLayout:
                new (&m_message_raw.keyboard_layout_name)
                    std::string(std::move(other.m_message_raw.keyboard_layout_name));
                break;
            case MessageType::WorkspaceActive:
                new (&m_message_raw.workspace_active_id) uint32_t(other.m_message_raw.workspace_active_id);
                break;
            case MessageType::WorkspaceList:
                new (&m_message_raw.workspace_list_ids) std::vector(other.m_message_raw.workspace_list_ids);
                break;
            case MessageType::RequestClientInit:
                break;
            }
        }
    }

    Message& operator=(Message&& other)
    {
        if (this != &other)
        {
            deinit();
            m_message_type = other.m_message_type;
            m_destroyed    = other.m_destroyed;
            if (!m_destroyed)
            {
                switch (m_message_type)
                {
                case MessageType::KeyboardLayout:
                    new (&m_message_raw.keyboard_layout_name)
                        std::string(std::move(other.m_message_raw.keyboard_layout_name));
                    break;
                case MessageType::WorkspaceActive:
                    new (&m_message_raw.workspace_active_id) uint32_t(other.m_message_raw.workspace_active_id);
                    break;
                case MessageType::WorkspaceList:
                    new (&m_message_raw.workspace_list_ids) std::vector(other.m_message_raw.workspace_list_ids);
                    break;
                case MessageType::RequestClientInit:
                    break;
                }
            }
        }
        return *this;
    }

    [[nodiscard]] MessageType message_type() const
    {
        Assert(!m_destroyed);
        return m_message_type;
    }

    template <MessageType E> [[nodiscard]] MessageTypeDataTypeT<E>* get()
    {
        Assert(!m_destroyed);
        Assert(m_message_type == E);
        if constexpr (E == MessageType::KeyboardLayout)
        {
            return &m_message_raw.keyboard_layout_name;
        }
        else if constexpr (E == MessageType::WorkspaceActive)
        {
            return &m_message_raw.workspace_active_id;
        }
        else if constexpr (E == MessageType::WorkspaceList)
        {
            return &m_message_raw.workspace_list_ids;
        }
        else if constexpr (E == MessageType::RequestClientInit)
        {
            return &m_message_raw.request_client_init;
        }
        Assert(false);
    }

    template <MessageType E> [[nodiscard]] const MessageTypeDataTypeT<E>* get() const
    {
        Assert(!m_destroyed);
        Assert(m_message_type == E);
        switch (E)
        {
        case MessageType::KeyboardLayout:
            return &m_message_raw.keyboard_layout_name;
        case MessageType::WorkspaceActive:
            return &m_message_raw.workspace_active_id;
        case MessageType::WorkspaceList:
            return &m_message_raw.workspace_list_ids;
        case MessageType::RequestClientInit:
            return &m_message_raw.request_client_init;
        }
        Assert(false);
    }

    // first 8 bytes - size, then 1 byte of the enum tag type, then (size - 1) of actual data
    void as_bytes(std::vector<char>& bytes) const
    {
        Assert(!m_destroyed);
        bytes.clear();
        bytes.resize(sizeof(size_t));
        bytes.push_back((char)m_message_type);

        switch (m_message_type)
        {
        case MessageType::KeyboardLayout:
        {
            auto& str = m_message_raw.keyboard_layout_name;
            bytes.insert(bytes.end(), str.data(), str.data() + str.size());
            break;
        }
        case MessageType::WorkspaceActive:
        {
            auto        id       = m_message_raw.workspace_active_id;
            const char* id_bytes = reinterpret_cast<const char*>(&id);
            bytes.insert(bytes.end(), id_bytes, id_bytes + sizeof(id));
            break;
        }
        case MessageType::WorkspaceList:
        {
            auto&  list_ids     = m_message_raw.workspace_list_ids;
            size_t len_list_ids = list_ids.size();

            const char* len_list_ids_bytes = reinterpret_cast<const char*>(&len_list_ids);
            bytes.insert(bytes.end(), len_list_ids_bytes, len_list_ids_bytes + sizeof(len_list_ids));

            const char* list_ids_bytes = reinterpret_cast<const char*>(list_ids.data());
            bytes.insert(bytes.end(), list_ids_bytes, list_ids_bytes + len_list_ids * sizeof(uint32_t));
            break;
        }
        case MessageType::RequestClientInit:
            break;
        }
        size_t size_bytes = bytes.size() - sizeof(size_t);
        std::memcpy(bytes.data(), &size_bytes, sizeof(size_t));
    }

    // first 1 byte of the enum tag type, then (size - 1) of actual data
    static std::optional<Message> from_bytes(const std::vector<char>& bytes)
    {
        if (bytes.size() < 1)
        {
            std::cerr << "bytes.size() < 1" << '\n';
            return std::nullopt;
        }
        auto        msg_type_maybe = (uint8_t)(bytes[0]);
        const char* data           = bytes.data() + 1;
        size_t      data_size      = bytes.size() - 1;

        switch (msg_type_maybe)
        {
        case (uint8_t)MessageType::KeyboardLayout:
        {
            std::string layout_name(data, data_size);
            return Message(tag<MessageType::KeyboardLayout>, std::move(layout_name));
        }
        case (uint8_t)MessageType::WorkspaceActive:
        {
            uint32_t workspace_id;
            if (data_size != sizeof(workspace_id))
            {
                std::cerr << "WorkspaceActive: data_size != sizeof(workspace_id)" << '\n';
                return std::nullopt;
            }
            std::memcpy(&workspace_id, data, sizeof(workspace_id));
            return Message(tag<MessageType::WorkspaceActive>, workspace_id);
        }
        case (uint8_t)MessageType::WorkspaceList:
        {
            if (data_size < sizeof(size_t))
            {
                std::cerr << "WorkspaceList: data_size < sizeof(size_t)" << '\n';
                return std::nullopt;
            }
            size_t len_workspaces;
            std::memcpy(&len_workspaces, data, sizeof(len_workspaces));

            if (data_size < len_workspaces * sizeof(uint32_t) + sizeof(size_t))
            {
                std::cerr << "WorkspaceList: data_size < len_workspaces * sizeof(uint32_t) + sizeof(size_t)" << '\n';
                return std::nullopt;
            }
            std::vector<uint32_t> workspaces{};
            workspaces.resize(len_workspaces);
            std::memcpy(workspaces.data(), data + sizeof(size_t), len_workspaces * sizeof(uint32_t));

            return Message(tag<MessageType::WorkspaceList>, std::move(workspaces));
        }
        default:
            std::cerr << "msg_type_maybe: " << msg_type_maybe << '\n';
            return std::nullopt;
        }
    }

  private:
    void deinit()
    {
        if (m_destroyed)
            return;
        switch (m_message_type)
        {
        case MessageType::KeyboardLayout:
            m_message_raw.keyboard_layout_name.~basic_string();
            break;
        case MessageType::WorkspaceActive:
            break;
        case MessageType::WorkspaceList:
            m_message_raw.workspace_list_ids.~vector();
            break;
        case MessageType::RequestClientInit:
            break;
        }
    }

  private:
    MessageType m_message_type;
    MessageRaw  m_message_raw;
    bool        m_destroyed = true;
};

#endif
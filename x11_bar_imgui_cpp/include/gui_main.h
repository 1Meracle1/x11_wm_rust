#ifndef AUDIO_APP
#define AUDIO_APP

#include <GLFW/glfw3.h>
#include <chrono>
#include <cstdint>
#include <fstream>
#include <xcb/xcb.h>
#include <sys/poll.h>

enum class ScreenLocation
{
    Top,
    Bottom,
};

struct Memory_Usage
{
    static constexpr const char* OS_FILE_NAME = "/proc/meminfo";
    std::ifstream                ifs{OS_FILE_NAME};

    static constexpr int64_t                           READ_WAIT_SECONDS = 1;
    std::chrono::time_point<std::chrono::steady_clock> last_read{};

    bool         first_read   = true;
    std::int64_t total_mb     = 0;
    std::int64_t used_mb      = 0;
    std::int64_t available_mb = 0;

    void update();
};

struct CPU_Usage
{
    static constexpr const char* OS_FILE_NAME = "/proc/stat";
    std::ifstream                ifs{OS_FILE_NAME};

    static constexpr int64_t                           READ_WAIT_SECONDS = 1;
    std::chrono::time_point<std::chrono::steady_clock> last_read{};

    bool      first_read = true;
    long long prev_user = 0, prev_nice = 0, prev_system = 0, prev_idle = 0;
    long long prev_iowait = 0, prev_irq = 0, prev_softirq = 0, prev_steal = 0;

    double total_usage = 0.0;

    void update();
};

class GUI_Main
{
  public:
    explicit GUI_Main(GLFWwindow* window, const char* font_path, float font_size, ScreenLocation screen_location);
    ~GUI_Main();

    void render();

  private:
    xcb_connection_t* m_xcb_conn = nullptr;
    int               m_xcb_fd   = 0;
    // pollfd            m_poll_fds[1];
    Memory_Usage m_memory_usage{};
    CPU_Usage    m_cpu_usage{};
};

#endif
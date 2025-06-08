#include "GLFW/glfw3.h"
#include "parse_num.hpp"
#include "slice.hpp"
#include "types.hpp"
#define GLFW_EXPOSE_NATIVE_X11
#include <GLFW/glfw3native.h>

#include "imgui.h"
#define IMGUI_USER_CONFIG "imgui_config.h"
#define IMGUI_ENABLE_FREETYPE

#include "gui_main.h"
#include "vulkan_renderer.h"

#include <chrono>

static void glfw_error_callback(int error, const char* description)
{
    fprintf(stderr, "GLFW Error %d: %s\n", error, description);
}

int main(int argc, char** argv)
{
#define Lit(str) Slice<const char>(str).chop_zero_termination()

    const char* font_path        = "";
    float       font_size        = 16.0f;
    auto        screen_location  = ScreenLocation::Top;
    int         padding_top      = 5;
    int         window_height    = 25;
    const char* unix_socket_path = "/tmp/x11_bar_imgui_cpp.socket";

    if (argc > 1)
    {
        for (int i = 1; i < argc; ++i)
        {
            auto arg = Slice((const char*)argv[i], strlen(argv[i]));
            if (arg == Lit("--help"))
            {
                std::cerr << "x11_bar, help options:\n"
                          << "\t--help      - to get help on possible command line arguments\n"
                          << "\t--font-path - to provide path to a font file\n"
                          << "\t--font-size - to provide font size\n";
                return 1;
            }
            if (arg == Lit("--font-path"))
            {
                if (i + 1 == argc)
                {
                    std::cerr << "no value provided for --font-path command line argument.\n";
                    return 1;
                }
                ++i;
                font_path = argv[i];
                continue;
            }
            if (arg == Lit("--font-size"))
            {
                if (i + 1 == argc)
                {
                    std::cerr << "no value provided for --font-size command line argument.\n";
                    return 1;
                }
                ++i;
                auto font_size_str = Slice((u8*)argv[i], strlen(argv[i]));
                auto res           = parse_float(font_size_str, font_size);
                if (res != ParseFloatFromStringError::Ok)
                {
                    std::cerr << "invalid value provided for --font-size command line argument.\n\tExpected positive "
                                 "non-zero floating "
                                 "point value, received '"
                              << font_size_str << "'\n";
                    return 1;
                }
                if (font_size < 1.0f)
                {
                    std::cerr << "invalid value provided for --font-size command line argument.\n\tExpected positive "
                                 "non-zero floating "
                                 "point value, received '"
                              << font_size_str << "'\n";
                    return 1;
                }
                continue;
            }
            if (arg == Lit("--location"))
            {
                if (i + 1 == argc)
                {
                    std::cerr << "no value provided for --location command line argument.\n";
                    return 1;
                }
                ++i;
                auto loc_str = Slice((const char*)argv[i], strlen(argv[i]));
                if (loc_str == Lit("top"))
                {
                    screen_location = ScreenLocation::Top;
                }
                else if (loc_str == Lit("bottom"))
                {
                    screen_location = ScreenLocation::Bottom;
                }
                else
                {
                    std::cerr << "incorrect value provided for --location command line argument.\n Should be either "
                                 "'top' or 'bottom', whereas received '"
                              << argv[i] << "'.\n";
                    return 1;
                }
                continue;
            }
        }
    }

    glfwSetErrorCallback(glfw_error_callback);
    if (!glfwInit())
        return 1;

    // Create window with Vulkan context
    glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
    glfwWindowHint(GLFW_VISIBLE, false);
    window_height      = std::max(window_height, (int)font_size + padding_top * 2);
    GLFWwindow* window = glfwCreateWindow(1920, (int)window_height, "X11 bar", nullptr, nullptr);
    if (!glfwVulkanSupported())
    {
        printf("GLFW: Vulkan Not Supported\n");
        return 1;
    }

    VulkanRenderer renderer(window);

    GUI_Main app{window, font_path, font_size, screen_location, window_height, unix_socket_path};

    ImVec4 clear_color = ImVec4(0.45f, 0.55f, 0.60f, 1.00f);

    constexpr double IDLE_TRESHOLD_SECONDS      = 2.0;
    constexpr int    IDLE_FPS                   = 15;
    bool             is_active_frame_rate_mode  = true;
    auto             last_interaction_timestamp = std::chrono::steady_clock::now();

    // Main loop
    while (!glfwWindowShouldClose(window))
    {
        // Poll and handle events (inputs, window resize, etc.)
        // You can read the io.WantCaptureMouse, io.WantCaptureKeyboard flags to tell if dear imgui wants to use your
        // inputs.
        // - When io.WantCaptureMouse is true, do not dispatch mouse input data to your main application, or
        // clear/overwrite your copy of the mouse data.
        // - When io.WantCaptureKeyboard is true, do not dispatch keyboard input data to your main application, or
        // clear/overwrite your copy of the keyboard data. Generally you may always pass all inputs to dear imgui, and
        // hide them from your application based on those two flags. ImGui::IsAnyItemActive()

        if (ImGui::IsAnyItemHovered() || ImGui::IsAnyItemFocused() || ImGui::IsAnyMouseDown())
        {
            last_interaction_timestamp = std::chrono::steady_clock::now();
            is_active_frame_rate_mode  = true;
        }
        else
        {
            auto                          now     = std::chrono::steady_clock::now();
            std::chrono::duration<double> elapsed = now - last_interaction_timestamp;
            if (elapsed.count() > IDLE_TRESHOLD_SECONDS)
                is_active_frame_rate_mode = false;
        }

        if (is_active_frame_rate_mode)
            glfwPollEvents();
        else
            glfwWaitEventsTimeout(1.0 / IDLE_FPS);

        if (renderer.beginFrame(window))
        {
            app.render();
            renderer.submitFrame(clear_color);
        }
    }

    glfwDestroyWindow(window);
    glfwTerminate();

    return 0;
}

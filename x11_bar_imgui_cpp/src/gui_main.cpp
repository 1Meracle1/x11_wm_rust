#include "gui_main.h"
#include "imgui.h"
#include "imgui_internal.h"
#include "message.hpp"
#include <cassert>
#include <ctime>
#include <iostream>
#include <sstream>
#include <xcb/xcb.h>
#define GLFW_EXPOSE_NATIVE_X11
#include <GLFW/glfw3native.h>
#include <X11/Xlib.h>
#include <X11/Xlib-xcb.h>
#include <X11/Xatom.h>
#include <xcb/xproto.h>
#include <chrono>
#include <string>

namespace
{
void setup_fonts(const char* font_path, float font_size)
{
    ImGuiIO& io = ImGui::GetIO();
    // Load Fonts
    // - If no fonts are loaded, dear imgui will use the default font. You can also load multiple fonts and use
    // ImGui::PushFont()/PopFont() to select them.
    // - AddFontFromFileTTF() will return the ImFont* so you can store it if you need to select the font among multiple.
    // - If the file cannot be loaded, the function will return a nullptr. Please handle those errors in your
    // application (e.g. use an assertion, or display an error and quit).
    // - The fonts will be rasterized at a given size (w/ oversampling) and stored into a texture when calling
    // ImFontAtlas::Build()/GetTexDataAsXXXX(), which ImGui_ImplXXXX_NewFrame below will call.
    // - Use '#define IMGUI_ENABLE_FREETYPE' in your imconfig file to use Freetype for higher quality font rendering.
    // - Read 'docs/FONTS.md' for more instructions and details.
    // - Remember that in C/C++ if you want to include a backslash \ in a string literal you need to write a double
    // backslash \\ !
    // io.Fonts->AddFontDefault();
    // io.Fonts->AddFontFromFileTTF("c:\\Windows\\Fonts\\segoeui.ttf", 18.0f);
    // static const ImWchar polishRanges[] = {
    //     0x0020, 0x00FF, // Basic Latin + Latin Supplement
    //     0x0100, 0x017F, // Latin Extended-A
    //     0x0180, 0x024F, // Latin Extended-B
    //     0x0370, 0x03FF, // Greek and Coptic
    //     0x2DE0, 0x2DFF, // Latin Extended Additional
    //     0,
    // };
    // ImFont* font =
    // io.Fonts->AddFontFromFileTTF("/usr/share/fonts/ubuntu/UbuntuMono-R.ttf", 18.0f, nullptr, polish_ranges);
    ImFont* font = nullptr;
    if (strcmp(font_path, "") != 0)
    {
        font = io.Fonts->AddFontFromFileTTF(font_path, font_size, nullptr);
    }
    else
    {
        font = io.Fonts->AddFontFromFileTTF("/usr/share/fonts/ubuntu/UbuntuMono-R.ttf", font_size, nullptr);
    }
    if (font == nullptr)
    {
        io.Fonts->AddFontDefault();
    }
    // io.Fonts->AddFontFromFileTTF("../../misc/fonts/Roboto-Medium.ttf", 16.0f);
    // io.Fonts->AddFontFromFileTTF("../../misc/fonts/Cousine-Regular.ttf", 15.0f);
    // ImFont* font = io.Fonts->AddFontFromFileTTF("c:\\Windows\\Fonts\\ArialUni.ttf", 18.0f, nullptr,
    // io.Fonts->GetGlyphRangesJapanese()); IM_ASSERT(font != nullptr);
}

void set_rfl_theme()
{
    ImGuiStyle& style = ImGui::GetStyle();

    style.FrameRounding   = 5.0f;
    style.FrameBorderSize = 1.0f;
    style.CellPadding     = ImVec2(4.0f, 5.0f);
    // style.FrameBorderSize = 1.5f;

    ImVec4* colors = ImGui::GetStyle().Colors;
    // colors[ImGuiCol_Text]                  = ImVec4(0.651f, 0.584f, 0.545f, 1.0f);
    colors[ImGuiCol_Text] = ImVec4(0.769f, 0.675f, 0.549f, 1.0f);
    // colors[ImGuiCol_Text]                  = ImVec4(0.90f, 0.90f, 0.90f, 1.00f);
    colors[ImGuiCol_TextDisabled]          = ImVec4(0.50f, 0.50f, 0.50f, 1.00f);
    colors[ImGuiCol_WindowBg]              = ImVec4(0.10f, 0.10f, 0.10f, 1.00f);
    colors[ImGuiCol_ChildBg]               = ImVec4(0.07f, 0.07f, 0.07f, 0.00f);
    colors[ImGuiCol_PopupBg]               = ImVec4(0.07f, 0.07f, 0.07f, 0.94f);
    colors[ImGuiCol_Border]                = ImVec4(0.24f, 0.25f, 0.25f, 0.59f);
    colors[ImGuiCol_BorderShadow]          = ImVec4(0.00f, 0.00f, 0.00f, 0.00f);
    colors[ImGuiCol_FrameBg]               = ImVec4(0.10f, 0.10f, 0.10f, 1.00f);
    colors[ImGuiCol_FrameBgHovered]        = ImVec4(0.10f, 0.10f, 0.10f, 1.00f);
    colors[ImGuiCol_FrameBgActive]         = ImVec4(0.42f, 0.32f, 0.22f, 1.00f);
    colors[ImGuiCol_TitleBg]               = ImVec4(0.20f, 0.20f, 0.20f, 1.00f);
    colors[ImGuiCol_TitleBgActive]         = ImVec4(0.42f, 0.32f, 0.22f, 1.00f);
    colors[ImGuiCol_TitleBgCollapsed]      = ImVec4(0.00f, 0.00f, 0.00f, 0.51f);
    colors[ImGuiCol_MenuBarBg]             = ImVec4(0.14f, 0.14f, 0.14f, 1.00f);
    colors[ImGuiCol_ScrollbarBg]           = ImVec4(0.02f, 0.02f, 0.02f, 0.53f);
    colors[ImGuiCol_ScrollbarGrab]         = ImVec4(0.31f, 0.31f, 0.31f, 1.00f);
    colors[ImGuiCol_ScrollbarGrabHovered]  = ImVec4(0.41f, 0.41f, 0.41f, 1.00f);
    colors[ImGuiCol_ScrollbarGrabActive]   = ImVec4(0.51f, 0.51f, 0.51f, 1.00f);
    colors[ImGuiCol_CheckMark]             = ImVec4(0.62f, 0.45f, 0.15f, 1.00f);
    colors[ImGuiCol_SliderGrab]            = ImVec4(0.62f, 0.45f, 0.15f, 1.00f);
    colors[ImGuiCol_SliderGrabActive]      = ImVec4(0.67f, 0.45f, 0.15f, 1.00f);
    colors[ImGuiCol_Button]                = ImVec4(0.16f, 0.16f, 0.16f, 1.00f);
    colors[ImGuiCol_ButtonHovered]         = ImVec4(0.23f, 0.22f, 0.20f, 1.00f);
    colors[ImGuiCol_ButtonActive]          = ImVec4(0.53f, 0.44f, 0.33f, 1.00f);
    colors[ImGuiCol_Header]                = ImVec4(0.30f, 0.30f, 0.30f, 1.00f);
    colors[ImGuiCol_HeaderHovered]         = ImVec4(0.31f, 0.26f, 0.19f, 0.80f);
    colors[ImGuiCol_HeaderActive]          = ImVec4(0.42f, 0.33f, 0.22f, 1.00f);
    colors[ImGuiCol_Separator]             = ImVec4(0.43f, 0.43f, 0.50f, 0.50f);
    colors[ImGuiCol_SeparatorHovered]      = ImVec4(0.42f, 0.18f, 0.13f, 0.78f);
    colors[ImGuiCol_SeparatorActive]       = ImVec4(0.42f, 0.18f, 0.13f, 1.00f);
    colors[ImGuiCol_ResizeGrip]            = ImVec4(0.42f, 0.18f, 0.13f, 0.20f);
    colors[ImGuiCol_ResizeGripHovered]     = ImVec4(0.42f, 0.18f, 0.13f, 0.67f);
    colors[ImGuiCol_ResizeGripActive]      = ImVec4(0.42f, 0.18f, 0.13f, 0.95f);
    colors[ImGuiCol_Tab]                   = ImVec4(0.20f, 0.20f, 0.20f, 0.98f);
    colors[ImGuiCol_TabHovered]            = ImVec4(0.37f, 0.32f, 0.22f, 0.96f);
    colors[ImGuiCol_TabActive]             = ImVec4(0.42f, 0.32f, 0.22f, 1.00f);
    colors[ImGuiCol_TabUnfocused]          = ImVec4(0.07f, 0.10f, 0.15f, 0.97f);
    colors[ImGuiCol_TabUnfocusedActive]    = ImVec4(0.14f, 0.26f, 0.42f, 1.00f);
    colors[ImGuiCol_PlotLines]             = ImVec4(0.61f, 0.61f, 0.61f, 1.00f);
    colors[ImGuiCol_PlotLinesHovered]      = ImVec4(1.00f, 0.43f, 0.35f, 1.00f);
    colors[ImGuiCol_PlotHistogram]         = ImVec4(0.90f, 0.70f, 0.00f, 1.00f);
    colors[ImGuiCol_PlotHistogramHovered]  = ImVec4(1.00f, 0.60f, 0.00f, 1.00f);
    colors[ImGuiCol_TableHeaderBg]         = ImVec4(0.19f, 0.19f, 0.20f, 1.00f);
    colors[ImGuiCol_TableBorderStrong]     = ImVec4(0.31f, 0.31f, 0.35f, 1.00f);
    colors[ImGuiCol_TableBorderLight]      = ImVec4(0.23f, 0.23f, 0.25f, 1.00f);
    colors[ImGuiCol_TableRowBg]            = ImVec4(0.00f, 0.00f, 0.00f, 0.00f);
    colors[ImGuiCol_TableRowBgAlt]         = ImVec4(1.00f, 1.00f, 1.00f, 0.06f);
    colors[ImGuiCol_TextSelectedBg]        = ImVec4(0.39f, 0.46f, 0.54f, 0.35f);
    colors[ImGuiCol_DragDropTarget]        = ImVec4(1.00f, 1.00f, 0.00f, 0.90f);
    colors[ImGuiCol_NavHighlight]          = ImVec4(0.42f, 0.18f, 0.13f, 1.00f);
    colors[ImGuiCol_NavWindowingHighlight] = ImVec4(1.00f, 1.00f, 1.00f, 0.70f);
    colors[ImGuiCol_NavWindowingDimBg]     = ImVec4(0.80f, 0.80f, 0.80f, 0.20f);
    colors[ImGuiCol_ModalWindowDimBg]      = ImVec4(0.80f, 0.80f, 0.80f, 0.35f);
}

void set_app_icon()
{
    // Application Icon
    // GLFWimage images[1];
    // images[0].pixels = stbi_load(std::format("{}\\{}", ROOT_DIR, R"(data\icons\3263182.png)").c_str(),
    // &images[0].width,
    //                              &images[0].height, 0, 4); // rgba channels
    // glfwSetWindowIcon(window, 1, images);
    // stbi_image_free(images[0].pixels);
}
} // namespace

GUI_Main::GUI_Main(GLFWwindow*    window,
                   const char*    font_path,
                   float          font_size,
                   ScreenLocation screen_location,
                   int            window_height,
                   const char*    wm_unix_socket_path)
    : m_unix_comm_bus(wm_unix_socket_path)
{
    setup_fonts(font_path, font_size);
    set_rfl_theme();
    set_app_icon();

    auto x11_display        = glfwGetX11Display();
    auto x11_window         = glfwGetX11Window(window);
    auto atom_strut_partial = XInternAtom(x11_display, "_NET_WM_STRUT_PARTIAL", 0);
    auto atom_window_type   = XInternAtom(x11_display, "_NET_WM_WINDOW_TYPE", 0);
    auto atom_dock          = XInternAtom(x11_display, "_NET_WM_WINDOW_TYPE_DOCK", 0);
    m_xcb_conn              = XGetXCBConnection(x11_display);
    m_xcb_fd                = xcb_get_file_descriptor(m_xcb_conn);
    // m_poll_fds[0].fd        = m_xcb_fd;
    // m_poll_fds[0].events    = POLLIN;

    // auto setup              = xcb_get_setup(xcb_conn);
    // auto iter               = xcb_setup_roots_iterator(setup);
    // auto screen             = iter.data;
    switch (screen_location)
    {
    case ScreenLocation::Top:
    {
        long strut[12] = {0, 0, (long)window_height, 0, 0, 0, 0, 0, 0, 0, 0, 0};
        XChangeProperty(x11_display, x11_window, atom_strut_partial, XA_CARDINAL, 32, PropModeReplace,
                        (const unsigned char*)(strut), 12);
        break;
    }
    case ScreenLocation::Bottom:
    {
        long strut[12] = {0, 0, 0, (long)window_height, 0, 0, 0, 0, 0, 0, 0, 0};
        XChangeProperty(x11_display, x11_window, atom_strut_partial, XA_CARDINAL, 32, PropModeReplace,
                        (const unsigned char*)(strut), 12);
        break;
    }
    }

    XChangeProperty(x11_display, x11_window, atom_window_type, XA_ATOM, 32, PropModeReplace,
                    (const unsigned char*)&atom_dock, 1);

    XMapWindow(x11_display, x11_window);
    XFlush(x11_display);

    auto              request_init_msg = Message(tag<MessageType::RequestClientInit>, RequestClientInit{});
    std::vector<char> bytes;
    request_init_msg.as_bytes(bytes);
    m_unix_comm_bus.notify_server(std::move(bytes));
}

GUI_Main::~GUI_Main() {}

void GUI_Main::render()
{
    m_memory_usage.update();
    m_cpu_usage.update();

    std::vector<char> bytes{};
    while (m_unix_comm_bus.try_pop_input_message(bytes))
    {
        if (auto message_maybe = Message::from_bytes(bytes); message_maybe.has_value())
        {
            auto& message = message_maybe.value();
            switch (message.message_type())
            {
            case MessageType::KeyboardLayout:
            {
                auto str_ptr           = message.get<MessageType::KeyboardLayout>();
                m_keyboard_layout_name = std::move(*str_ptr);
                break;
            }
            case MessageType::WorkspaceList:
            {
                auto workspaces = message.get<MessageType::WorkspaceList>();
                m_workspaces    = std::move(*workspaces);

                std::stringstream ss;
                ss << "updated workspaces: [";
                for (size_t i = 0; i < m_workspaces.size(); ++i)
                {
                    ss << m_workspaces[i];
                    if (i + 1 < m_workspaces.size())
                    {
                        ss << ", ";
                    }
                }
                ss << "]\n";
                std::cout << ss.rdbuf();

                break;
            }
            case MessageType::WorkspaceActive:
            {
                auto active_workspace = message.get<MessageType::WorkspaceActive>();
                m_active_workspace    = *active_workspace;
                std::cout << "updated active worskpace: " << m_active_workspace << '\n';
                break;
            }
            case MessageType::RequestClientInit:
                break;
            }
        }
    }

    // Our state
    // static bool show_demo_window = false;

    // Set the next window as a full screen
    static ImGuiWindowFlags flags =
        ImGuiWindowFlags_NoDecoration | ImGuiWindowFlags_NoMove | ImGuiWindowFlags_NoBringToFrontOnFocus;
    const ImGuiViewport* viewport = ImGui::GetMainViewport();
    ImGui::SetNextWindowPos(viewport->WorkPos);
    ImGui::SetNextWindowSize(viewport->WorkSize);
    ImVec2 center = viewport->GetCenter();

    if (ImGui::Begin("Fullscreen main window", nullptr, flags))
    {
        static float width = 200.0f;
        ImGui::SetCursorPosX(center.x - width / 2.0f);
        ImGui::SetCursorPosY(5);

        width = ImGui::GetItemRectSize().x;
        {
            ImGui::BeginGroup();

            const float text_height = ImGui::GetTextLineHeight();
            ImGui::Dummy(ImVec2(0.0f, text_height));
            ImGui::SameLine();
            ImGui::SeparatorEx(ImGuiSeparatorFlags_Vertical);

            for (auto workspace : m_workspaces)
            {
                ImGui::SameLine();
                if (workspace == m_active_workspace)
                {
                    ImGui::Text("[ %d ]", workspace);
                }
                else
                {
                    ImGui::Text("  %d  ", workspace);
                }
                ImGui::SameLine();
                ImGui::SeparatorEx(ImGuiSeparatorFlags_Vertical);
            }
            ImGui::SameLine();
            ImGui::Dummy(ImVec2(50.0f, text_height));
            ImGui::SameLine();

            // ImGui::SeparatorEx(ImGuiSeparatorFlags_Vertical);
            // ImGui::SameLine();
            // ImGui::Text("FPS: %.1f", (double)ImGui::GetIO().Framerate);
            // ImGui::SameLine();

            ImGui::SeparatorEx(ImGuiSeparatorFlags_Vertical);
            ImGui::SameLine();
            ImGui::Text("CPU: %3d%%", (int)m_cpu_usage.total_usage);
            ImGui::SameLine();
            ImGui::SeparatorEx(ImGuiSeparatorFlags_Vertical);
            ImGui::SameLine();
            ImGui::Text("Memory: %ld", m_memory_usage.used_mb);
            ImGui::SameLine();
            ImGui::SeparatorEx(ImGuiSeparatorFlags_Vertical);

            if (!m_keyboard_layout_name.empty())
            {
                ImGui::SameLine();
                ImGui::Text("Lang: %s", m_keyboard_layout_name.data());
                ImGui::SameLine();
                ImGui::SeparatorEx(ImGuiSeparatorFlags_Vertical);
            }

            {
                ImGui::SameLine();
                if (m_alsa.is_muted())
                {
                    ImGui::Text("Vol: Muted");
                }
                else
                {
                    ImGui::Text("Vol: %3d%%", (int)m_alsa.current_volume_percentage());
                }
                if (ImGui::IsItemClicked())
                {
                    m_alsa.mute_toggle();
                }
                if (ImGui::IsItemHovered())
                {
                    auto wheel = ImGui::GetIO().MouseWheel;
                    if (wheel > 0)
                    {
                        m_alsa.increase_volume();
                    }
                    else if (wheel < 0)
                    {
                        m_alsa.decrease_volume();
                    }
                }
                ImGui::SameLine();
                ImGui::SeparatorEx(ImGuiSeparatorFlags_Vertical);
            }

            // ImGui::SameLine();
            // ImGui::Dummy(ImVec2(100.0f, text_height));
            // ImGui::SameLine();
            // ImGui::SeparatorEx(ImGuiSeparatorFlags_Vertical);

            {
                std::time_t time_now = std::time(nullptr);
                std::tm*    local_tm = std::localtime(&time_now);
                char        date_buf[40];
                char        time_buf[20];
                std::strftime(date_buf, sizeof(date_buf), "%a, %d %b %Y", local_tm);
                std::strftime(time_buf, sizeof(time_buf), "%I:%M %p", local_tm);

                ImGui::SameLine();
                ImGui::Text("%s", date_buf);
                ImGui::SameLine();
                ImGui::SeparatorEx(ImGuiSeparatorFlags_Vertical);

                ImGui::SameLine();
                ImGui::Text("%s", time_buf);
                ImGui::SameLine();
                ImGui::SeparatorEx(ImGuiSeparatorFlags_Vertical);
            }

            ImGui::EndGroup();
        }
        width = ImGui::GetItemRectSize().x;
        // if (ImGui::Button(show_demo_window ? "Hide Demo window" : "Show Demo window"))
        //     show_demo_window = !show_demo_window;
        // if (show_demo_window)
        //     ImGui::ShowDemoWindow(&show_demo_window);

        ImGui::End();
    }
}

void Memory_Usage::update()
{
    if (not first_read)
    {
        auto                          now  = std::chrono::steady_clock::now();
        std::chrono::duration<double> diff = now - last_read;
        if (diff.count() < READ_WAIT_SECONDS)
        {
            return;
        }
    }
    if (!ifs.is_open())
    {
        ifs.open(OS_FILE_NAME);
    }
    if (ifs.is_open())
    {
        ifs.clear();
        ifs.seekg(0);

        long long   mem_total     = -1;
        long long   mem_available = -1;
        std::string line;
        int         lines_read = 0;
        while (lines_read < 5 && std::getline(ifs, line))
        {
            if (line.rfind("MemTotal:", 0) == 0)
            {
                sscanf(line.c_str(), "MemTotal: %lld kB", &mem_total);
            }
            else if (line.rfind("MemAvailable:", 0) == 0)
            {
                sscanf(line.c_str(), "MemAvailable: %lld kB", &mem_available);
            }
            // If we have both values, we can stop parsing
            if (mem_total != -1 && mem_available != -1)
            {
                break;
            }
            lines_read++;
        }

        if (mem_total != -1 && mem_available != -1)
        {
            total_mb     = mem_total / 1024;
            available_mb = mem_available / 1024;
            used_mb      = total_mb - available_mb;
        }
    }
    if (first_read)
    {
        first_read = false;
    }
    last_read = std::chrono::steady_clock::now();
}

void CPU_Usage::update()
{
    if (not first_read)
    {
        auto                          now  = std::chrono::steady_clock::now();
        std::chrono::duration<double> diff = now - last_read;
        if (diff.count() < READ_WAIT_SECONDS)
        {
            return;
        }
    }
    if (!ifs.is_open())
    {
        ifs.open(OS_FILE_NAME);
    }
    if (ifs.is_open())
    {
        ifs.clear();
        ifs.seekg(0);

        std::string line;
        std::getline(ifs, line);

        long long user, nice, system, idle, iowait, irq, softirq, steal;
        sscanf(line.c_str(), "cpu %lld %lld %lld %lld %lld %lld %lld %lld", &user, &nice, &system, &idle, &iowait, &irq,
               &softirq, &steal);

        if (first_read)
        {
            prev_user    = user;
            prev_nice    = nice;
            prev_system  = system;
            prev_idle    = idle;
            prev_iowait  = iowait;
            prev_irq     = irq;
            prev_softirq = softirq;
            prev_steal   = steal;
        }
        else
        {

            long long prev_idle_total = prev_idle + prev_iowait;
            long long idle_total      = idle + iowait;

            long long prev_non_idle = prev_user + prev_nice + prev_system + prev_irq + prev_softirq + prev_steal;
            long long non_idle      = user + nice + system + irq + softirq + steal;

            long long prev_total = prev_idle_total + prev_non_idle;
            long long total      = idle_total + non_idle;

            long long total_delta = total - prev_total;
            long long idle_delta  = idle_total - prev_idle_total;

            // Store current values for the next calculation
            prev_user    = user;
            prev_nice    = nice;
            prev_system  = system;
            prev_idle    = idle;
            prev_iowait  = iowait;
            prev_irq     = irq;
            prev_softirq = softirq;
            prev_steal   = steal;

            if (total_delta == 0)
            {
                total_usage = 0.0;
            }
            else
            {
                total_usage = (double)(total_delta - idle_delta) / (double)total_delta * 100.0;
            }
        }
    }
    if (first_read)
    {
        first_read = false;
    }
    last_read = std::chrono::steady_clock::now();
}
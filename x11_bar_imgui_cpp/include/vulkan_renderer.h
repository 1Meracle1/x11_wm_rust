#ifndef PLATFORM_H
#define PLATFORM_H

#include "imgui_impl_vulkan.h"
#define GLFW_INCLUDE_NONE
#define GLFW_INCLUDE_VULKAN
#include <GLFW/glfw3.h>

class VulkanRenderer {
public:
  explicit VulkanRenderer(GLFWwindow* window);
  ~VulkanRenderer();

  // returns whether to go futher or skip loop iteration
  bool beginFrame(GLFWwindow *window);
  
  void submitFrame(const ImVec4& clear_color);

private:
    ImGui_ImplVulkanH_Window *wd;
};

#endif
# Example usage:
#  mkdir build
#  cd build
#  cmake -g "Visual Studio 14 2015" ..

cmake_minimum_required(VERSION 3.28)

project(x11_bar_imgui_cpp)

set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_CXX_COMPILER_LAUNCHER ccache)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release CACHE STRING "Choose the type of build." FORCE)
endif()

set(WARN_FLAGS 
  -Wall
  -Wextra # reasonable and standard
  -Wshadow # warn the user if a variable declaration shadows one from a parent context
  -Wnon-virtual-dtor # warn the user if a class with virtual functions has a non-virtual destructor. This helps
  # catch hard to track down memory errors
  -Wold-style-cast # warn for c-style casts
  -Wcast-align # warn for potential performance problem casts
  -Wunused # warn on anything being unused
  -Woverloaded-virtual # warn if you overload (not override) a virtual function
  -Wpedantic # warn if non-standard C++ is used
  -Wconversion # warn on type conversions that may lose data
  -Wsign-conversion # warn on sign conversions
  -Wnull-dereference # warn if a null dereference is detected
  -Wdouble-promotion # warn if float is implicit promoted to double
  -Wformat=2 # warn on security issues around functions that format output (ie printf)
  -Wimplicit-fallthrough # warn on statements that fallthrough without an explicit annotation
  -Wno-old-style-cast
)

set(COMPILE_OPTIONS
  ${WARN_FLAGS}
  -fno-rtti -fno-exceptions # baseline
)
set(LINK_OPTIONS 
  # -fuse-ld=lld
  -fuse-ld=mold
)

# set(LINKED_LIBS -lstdc++)

if (CMAKE_BUILD_TYPE STREQUAL "Debug")
    message(STATUS "Debug build")
    list(APPEND COMPILE_OPTIONS 
      -fsanitize=address,leak,undefined
      -fno-omit-frame-pointer
      -O0 
      -g
    )
    list(APPEND LINK_OPTIONS
      -fsanitize=address,leak,undefined
    )
    list(APPEND DEFINES 
        DEBUG_BUILD=1
    )
    list(APPEND LINKED_LIBS
        -lunwind -lunwind-x86_64 -lc++abi
    )
elseif (CMAKE_BUILD_TYPE STREQUAL "Release")
    message(STATUS "Release build")
    list(APPEND COMPILE_OPTIONS
      -O3 
      -fno-omit-frame-pointer 
      -march=native
      -flto
      -ftree-vectorize
    )
    list(APPEND DEFINES 
        RELEASE_BUILD=1
    )
# elseif (CMAKE_BUILD_TYPE STREQUAL "RelWithDebInfo")
#     message(STATUS "Release with Debug Info build")
#     list(APPEND COMPILE_OPTIONS 
#       -fno-omit-frame-pointer 
#       -g 
#       -O3 
#       -march=native
#     )
# elseif (CMAKE_BUILD_TYPE STREQUAL "MinSizeRel")
#     message(STATUS "Minimum Size Release build")
#     list(APPEND COMPILE_OPTIONS 
#       -Os 
#       -march=native
#     )
endif()

list(APPEND DEFINES VK_PROTOTYPES)
# set (CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -DVK_PROTOTYPES")
# set (CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -DVK_PROTOTYPES")

set(LIBS_DIR external)

# GLFW
set(GLFW_DIR ${LIBS_DIR}/glfw) # Set this to point to an up-to-date GLFW repo
option(GLFW_BUILD_EXAMPLES "Build the GLFW example programs" OFF)
option(GLFW_BUILD_TESTS "Build the GLFW test programs" OFF)
option(GLFW_BUILD_DOCS "Build the GLFW documentation" OFF)
option(GLFW_INSTALL "Generate installation target" OFF)
option(GLFW_DOCUMENT_INTERNALS "Include internals in documentation" OFF)
add_subdirectory(${GLFW_DIR} binary_dir EXCLUDE_FROM_ALL)
# include_directories(${GLFW_DIR}/include)

set(IMGUI_DIR ${LIBS_DIR}/imgui)

find_package(Vulkan REQUIRED)

list(APPEND LINKED_LIBS
  glfw 
  Vulkan::Vulkan
)

find_package(Freetype REQUIRED)

find_package(TBB REQUIRED)

add_executable(${PROJECT_NAME} 
  src/main.cpp
  src/vulkan_renderer.cpp
  src/gui_main.cpp
  src/unix_socket.cpp

  ${IMGUI_DIR}/backends/imgui_impl_glfw.cpp ${IMGUI_DIR}/backends/imgui_impl_vulkan.cpp ${IMGUI_DIR}/imgui.cpp ${IMGUI_DIR}/imgui_draw.cpp ${IMGUI_DIR}/imgui_demo.cpp ${IMGUI_DIR}/imgui_tables.cpp ${IMGUI_DIR}/imgui_widgets.cpp
  ${IMGUI_DIR}/misc/freetype/imgui_freetype.cpp
  ${IMGUI_DIR}/misc/cpp/imgui_stdlib.cpp
)

# Use vulkan headers from glfw:
target_include_directories(${PROJECT_NAME} PUBLIC 
  ${GLFW_DIR}/include
  ${GLFW_DIR}/deps
  ${LIBS_DIR}
  
  ${IMGUI_DIR} 
  ${IMGUI_DIR}/backends
  ${IMGUI_DIR}/misc/cpp/

  ${FREETYPE_INCLUDE_DIRS}
  include
)

target_compile_definitions(${PROJECT_NAME} PRIVATE ${DEFINES})
target_compile_options(${PROJECT_NAME} PRIVATE ${COMPILE_OPTIONS})
target_link_options(${PROJECT_NAME} PRIVATE ${LINK_OPTIONS})
target_link_libraries(${PROJECT_NAME} PRIVATE ${LINKED_LIBS} ${FREETYPE_LIBRARIES} X11 xcb X11-xcb TBB::tbb asound)


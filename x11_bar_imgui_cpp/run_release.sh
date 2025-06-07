#!/bin/fish

# cmake -DCMAKE_BUILD_TYPE:STRING=Release -S . -B build/release
cmake --build build/release --target x11_bar_imgui_cpp -j 12 --parallel || exit
build/x11_bar_imgui_cpp
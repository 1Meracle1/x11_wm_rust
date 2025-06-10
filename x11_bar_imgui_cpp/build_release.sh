#!/bin/fish

cmake --build build/release --target x11_bar_imgui_cpp -j 12 --parallel || exit
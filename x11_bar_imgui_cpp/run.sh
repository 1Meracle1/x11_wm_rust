#!/bin/fish

cmake --build build --target x11_bar_imgui_cpp -j 12 --parallel || exit
build/x11_bar_imgui_cpp
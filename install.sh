#!/bin/fish

# window manager
cargo build -p window_manager --release || exit

set wm_target_path "/usr/local/bin/x11_wm_rust"
if test -d "$wm_target_path"
    sudo rm "$wm_target_path"
end
sudo cp target/release/window_manager "$wm_target_path" || exit

# bar
pushd x11_bar_imgui_cpp
./build_release.sh || exit

set bar_target_path "/usr/local/bin/x11_bar_cpp"
if test -d "$bar_target_path"
    sudo rm "$bar_target_path"
end
sudo cp build/release/x11_bar_imgui_cpp "$bar_target_path" || exit

popd || exit

# configs
set config_dir "$HOME/.config/x11_wm_rust"
if test -d "$config_dir"
    rm -rf "$config_dir"
end
echo "creating config directory"
mkdir -p "$config_dir"
cp -r data/* "$config_dir/"
cp config.toml "$config_dir/config.toml"
# if not test -d "$config_dir"
#     echo "creating config directory"
#     mkdir -p "$config_dir"
#     cp -r data/* "$config_dir/"
#     cp config.toml "$config_dir/config.toml"
# else
#     echo "skipping creation of config directory '$config_dir' as it already exists."
# end

# SDDM stuff
sudo rm /usr/local/bin/x11_wm_rust_session
sudo rm /usr/share/xsessions/x11_wm_rust.desktop
sudo cp x11_wm_rust_session /usr/local/bin/x11_wm_rust_session || exit
sudo cp x11_wm_rust.desktop /usr/share/xsessions/x11_wm_rust.desktop || exit

# sudo systemctl restart sddm || exit
#!/bin/fish

cargo build --manifest-path window_manager/Cargo.toml 
cargo build --manifest-path wm_msg/Cargo.toml 

set XEPHYR $(whereis -b Xephyr | sed -E 's/^.*: ?//')
if [ -z "$XEPHYR" ]
  echo "Xephyr not found"
  exit 1
end

xinit ./window_manager/xinitrc_debug -- "$XEPHYR" :100 -ac -screen 1920x1080 -host-cursor

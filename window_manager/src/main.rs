mod config;
mod ewmh;
mod icccm;
mod keybindings;
mod monitor;
mod unix_server;
mod window;
mod workspace;
mod xcb_wm;

use std::sync::mpsc;

use env_logger::Env;
use log::{debug, trace};
use msg_types::WmCommand;
use unix_server::UnixServerWorker;

const WM_NAME: &'static str = "Unknown WM";

fn main() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "debug")
        .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let config = config::Config::new("config.toml");
    // debug!("config: {:#?}", config);
    trace!("Loaded config.");

    let (sender, receiver) = mpsc::channel();
    let _unix_worker = UnixServerWorker::new("/tmp/x11_wm_rust", sender);

    let mut wm = xcb_wm::XcbWindowManager::new(WM_NAME, &config);
    trace!("Initialized XCB Window Manager.");

    wm.execute_startup_commands(&config);
    trace!("Executed startup commands.");

    loop {
        receiver.try_iter().for_each(|msg| {
            debug!("message received via Unix domain socket: {:?}", msg);
            match msg {
                WmCommand::FocusLeft => wm.handle_shift_focus_left(),
                WmCommand::FocusRight => wm.handle_shift_focus_right(),
                WmCommand::MoveLeft => wm.handle_window_move_left(&config),
                WmCommand::MoveRight => wm.handle_window_move_right(&config),
                WmCommand::WorkspaceChange(workspace_id) => {
                    wm.handle_change_workspace_id(workspace_id, &config)
                }
            }
        });
        if !wm.handle_event(&config) {
            break;
        }
    }
}

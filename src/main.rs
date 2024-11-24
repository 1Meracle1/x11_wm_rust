pub mod config;
pub mod ewmh;
pub mod icccm;
pub mod keybindings;
pub mod monitor;
pub mod window;
pub mod workspace;
pub mod xcb_wm;

use env_logger::Env;
use log::{debug, trace};

const WM_NAME: &'static str = "Unknown WM";

fn main() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "debug")
        .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let config = config::Config::new("config.toml");
    debug!("config: {:#?}", config);
    trace!("Loaded config.");

    let mut wm = xcb_wm::XcbWindowManager::new(WM_NAME, &config);
    trace!("Initialized XCB Window Manager.");

    wm.execute_startup_commands(&config);
    trace!("Executed startup commands.");

    while wm.handle_event(&config) {}
}

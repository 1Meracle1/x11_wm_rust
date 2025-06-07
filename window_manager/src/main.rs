use std::{fs::File, io::Write, os::unix::net::UnixStream, path::Path};

use config::{Config, ConfigErrors};
use env_logger::Env;
use keybindings::{
    execute_command_from_str, handle_key_press, keybindings_from_config, keybindings_grab,
};
use log::{error, info, warn};
use monitor::Monitor;
use x11_bindings::{
    bindings::{
        XCB_CW_EVENT_MASK, XCB_EVENT_MASK_SUBSTRUCTURE_NOTIFY, XCB_EVENT_MASK_SUBSTRUCTURE_REDIRECT,
    },
    connection::{self, Connection, MouseButton},
};

use crate::keybindings::keybindings_ungrab;

mod config;
mod keybindings;
mod monitor;
mod window;
mod workspace;

fn main() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let pid = std::process::id();
    info!("PID: {}", pid);

    let mut config = Config::new("config.toml")
        .or_else(|err| {
            error!("Failed init config from file, error: {:?}", err);
            warn!("Initializing config from default values.");
            Ok::<Config, ConfigErrors>(Config::default())
        })
        .unwrap();

    let conn = Connection::new().unwrap();

    if let Err(err) = conn.change_window_attrs_checked(
        conn.root(),
        XCB_CW_EVENT_MASK,
        XCB_EVENT_MASK_SUBSTRUCTURE_REDIRECT | XCB_EVENT_MASK_SUBSTRUCTURE_NOTIFY,
    ) {
        error!("Failed to acquire control over the root window.");
        error!("Error details: {}", err);
        error!("Another window manager already running, exiting.");
        return;
    }

    let mut monitor = Monitor::new(&conn);

    for cmd_str in &config.startup_commands {
        execute_command_from_str(cmd_str);
    }

    let mut keybindings = keybindings_from_config(&config);
    keybindings_grab(&keybindings, &conn);
    // trace!("keybindings: {:#?}", keybindings);

    conn.change_cursor("left_ptr");
    conn.grab_button(MouseButton::Left);

    const BAR_SOCKET_PATH: &str = "/tmp/x11_bar_imgui_cpp.socket";
    let bar_socket = Path::new(BAR_SOCKET_PATH);
    if !bar_socket.exists() {
        File::create(BAR_SOCKET_PATH).expect(
            format!(
                "failed to create bar socket file under path: {}",
                BAR_SOCKET_PATH
            )
            .as_str(),
        );
    }
    let mut bar_unix_stream: Option<UnixStream> = UnixStream::connect(bar_socket)
        .map(|s| Some(s))
        .map_or(None, |_| None);

    conn.flush();

    let mut requested_config_reload = false;
    loop {
        if let Some(event_res) = conn.wait_for_event() {
            match event_res {
                Ok(event) => match event {
                    connection::XcbEvents::KeyPress { modifier, keycode } => handle_key_press(
                        &keybindings,
                        &conn,
                        &config,
                        &mut monitor,
                        modifier,
                        keycode,
                        &mut requested_config_reload,
                    ),
                    connection::XcbEvents::MapRequst { window } => {
                        monitor.handle_map_request(&conn, &config, window)
                    }
                    connection::XcbEvents::FocusIn { window, mode } => {
                        monitor.handle_focus_in(&conn, &config, window, mode)
                    }
                    connection::XcbEvents::FocusOut { window, mode } => {
                        monitor.handle_focus_out(&conn, &config, window, mode)
                    }
                    connection::XcbEvents::EnterNotify { window } => {
                        monitor.handle_enter_notify(window, &conn, &config);
                    }
                    connection::XcbEvents::LeaveNotify { window: _ } => {}
                    connection::XcbEvents::ButtonPress {
                        x,
                        y,
                        window,
                        state,
                        detail,
                    } => {
                        monitor.handle_button_press(x, y, window, state, detail, &conn, &config);
                    }
                    connection::XcbEvents::ButtonRelease { x: _, y: _ } => {
                        monitor.handle_button_release(&conn);
                    }
                    connection::XcbEvents::MotionNotify {
                        x,
                        y,
                        window,
                        state,
                    } => {
                        monitor.handle_motion_notify(x, y, window, state, &conn, &config);
                    }
                    connection::XcbEvents::XkbStateNotify { event } => {
                        let group_idx = unsafe { *event }.group as usize;
                        let keyboard_layout_names = conn.xkb_get_layout_names();
                        // info!("keyboard_layout_names: {:?}", keyboard_layout_names);
                        if let Some(keyboard_layout_name) = keyboard_layout_names.get(group_idx) {
                            info!("keyboard_layout_name: {}", keyboard_layout_name);

                            let msg = format!("keyboard_layout_name: {}", keyboard_layout_name);
                            if let Some(bar_unix_stream) = &mut bar_unix_stream {
                                match bar_unix_stream.write_all(&msg.len().to_ne_bytes()) {
                                    Ok(_) => {
                                        if let Err(err) = bar_unix_stream.write_all(msg.as_bytes())
                                        {
                                            warn!("failed to send message to the bar: {}", err);
                                        }
                                    }
                                    Err(err) => {
                                        warn!("failed to send message length to the bar: {}", err)
                                    }
                                };
                            }
                        }
                    }
                },
                Err(error) => warn!("Error event: {:?}", error),
            };
        }
        monitor.check_deleted(&conn);

        if requested_config_reload {
            requested_config_reload = false;
            info!("requested config reload");

            match Config::new("config.toml") {
                Ok(new_config) => {
                    info!("updated config parsed successfully: {:#?}", new_config);

                    if config.startup_commands != new_config.startup_commands {
                        let diff = new_config.startup_commands.iter().filter(|exist_cmd| {
                            config
                                .startup_commands
                                .iter()
                                .find(|new_cmd| new_cmd == exist_cmd)
                                .is_none()
                        });
                        for cmd_str in diff {
                            execute_command_from_str(cmd_str);
                        }
                    }

                    let new_keybindings = keybindings_from_config(&config);
                    if keybindings != new_keybindings {
                        keybindings_ungrab(&keybindings, &conn);
                        keybindings = new_keybindings;
                        keybindings_grab(&keybindings, &conn);
                    }

                    monitor.remap_windows_with_upd_config(&conn, &config, &new_config);
                    config = new_config;
                }
                Err(err) => error!("Failed to reload config from file, error: {:?}", err),
            }
        }
    }
}

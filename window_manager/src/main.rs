use std::path::Path;

use config::{Config, ConfigErrors};
use env_logger::Env;
use keybindings::{
    execute_command_from_str, handle_key_press, keybindings_from_config, keybindings_grab,
};
use log::{error, info, trace, warn};
use monitor::Monitor;
use x11_bindings::{
    bindings::{
        XCB_CW_EVENT_MASK, XCB_EVENT_MASK_SUBSTRUCTURE_NOTIFY, XCB_EVENT_MASK_SUBSTRUCTURE_REDIRECT,
    },
    connection::{self, Connection, MouseButton},
    epoll::Epoll,
    inotify::Inotify,
};

use crate::{
    bar_message::{BarCommsBus, Message},
    keybindings::{execute_command_from_str_wait, keybindings_ungrab},
};

mod bar_message;
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
    let mut inotify = Inotify::new().expect("failed to create inotify instance");
    inotify
        .add_watch("config.toml")
        .expect("failed to add config file to watch list");

    let conn = Connection::new().unwrap();
    let x11_conn_fd = conn.get_file_descriptor();

    let mut epoll = Epoll::new(10).expect("failed to create epoll instance");
    epoll
        .add_watch(x11_conn_fd)
        .expect("failed to add x11 connection fd to epoll watch list");
    epoll
        .add_watch(inotify.fd)
        .expect("failed to add inotify fd to epoll watch list");

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
        if cmd_str.contains(" && ") {
            let parts = cmd_str.split(" && ").collect::<Vec<_>>();
            parts
                .iter()
                .take(parts.len() - 1)
                .for_each(|cmd_str| execute_command_from_str_wait(cmd_str));
            execute_command_from_str(parts.last().unwrap());
        } else {
            execute_command_from_str(cmd_str);
        }
    }

    let mut keybindings = keybindings_from_config(&config);
    keybindings_grab(&keybindings, &conn);
    // trace!("keybindings: {:#?}", keybindings);

    conn.change_cursor("left_ptr");
    conn.grab_button(MouseButton::Left);

    const BAR_SOCKET_PATH_STR: &str = "/tmp/x11_bar_imgui_cpp.socket";
    let bar_socket_path = Path::new(BAR_SOCKET_PATH_STR);
    let mut bar_comms_bus = BarCommsBus::new(bar_socket_path);

    conn.flush();

    trace!("started the loop");
    let mut requested_config_reload = false;
    loop {
        let events = epoll
            .wait()
            .expect("epoll failed while waiting for new events");
        for event in events {
            if event.u64 == x11_conn_fd as u64 {
                while let Some(event_res) = conn.poll_for_event() {
                    match event_res {
                        Ok(event) => match event {
                            connection::XcbEvents::KeyPress { modifier, keycode } => {
                                handle_key_press(
                                    &keybindings,
                                    &conn,
                                    &config,
                                    &mut monitor,
                                    modifier,
                                    keycode,
                                    &mut requested_config_reload,
                                    &mut bar_comms_bus,
                                )
                            }
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
                                monitor.handle_button_press(
                                    x, y, window, state, detail, &conn, &config,
                                );
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
                                if let Some(keyboard_layout_name) =
                                    keyboard_layout_names.get(group_idx)
                                {
                                    // trace!("keyboard_layout_name: {}", keyboard_layout_name);
                                    let message = Message::KeyboardLayout(keyboard_layout_name);
                                    bar_comms_bus.send_message(message);
                                }
                            }
                        },
                        Err(error) => warn!("Error event: {:?}", error),
                    };
                }
                monitor.check_deleted(&conn);
                conn.flush();
            } else if event.u64 == inotify.fd as u64 {
                match inotify.read_event() {
                    Ok(event) => match event {
                        x11_bindings::inotify::InotifyEvent::Modify => {
                            trace!("config file was updated, reloading...");
                            match Config::new("config.toml") {
                                Ok(new_config) => {
                                    trace!("updated config parsed successfully: {:#?}", new_config);

                                    if config.startup_commands != new_config.startup_commands {
                                        let diff = new_config.startup_commands.iter().filter(
                                            |exist_cmd| {
                                                config
                                                    .startup_commands
                                                    .iter()
                                                    .find(|new_cmd| new_cmd == exist_cmd)
                                                    .is_none()
                                            },
                                        );
                                        for cmd_str in diff {
                                            if cmd_str.contains(" && ") {
                                                let parts =
                                                    cmd_str.split(" && ").collect::<Vec<_>>();
                                                parts.iter().take(parts.len() - 1).for_each(
                                                    |cmd_str| {
                                                        execute_command_from_str_wait(cmd_str)
                                                    },
                                                );
                                                execute_command_from_str(parts.last().unwrap());
                                            } else {
                                                execute_command_from_str(cmd_str);
                                            }
                                        }
                                    }

                                    let new_keybindings = keybindings_from_config(&config);
                                    if keybindings != new_keybindings {
                                        keybindings_ungrab(&keybindings, &conn);
                                        keybindings = new_keybindings;
                                        keybindings_grab(&keybindings, &conn);
                                    }

                                    monitor.remap_windows_with_upd_config(
                                        &conn,
                                        &config,
                                        &new_config,
                                    );
                                    config = new_config;
                                }
                                Err(err) => {
                                    error!("Failed to reload config from file, error: {:?}", err)
                                }
                            }
                        }
                        _ => {
                            trace!("other inotify event: {:?}", event);
                        }
                    },
                    Err(err) => warn!("inotify read error: {:?}", err),
                }
            }
        }
    }
}

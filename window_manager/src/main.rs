use std::{
    io::Write,
    os::{fd::AsRawFd, unix::net::UnixListener},
    path::Path,
};

use config::{Config, ConfigErrors};
use env_logger::Env;
use keybindings::{
    execute_command_from_str, handle_key_press, keybindings_from_config, keybindings_grab,
};
use log::{error, trace, warn};
use monitor::Monitor;
use x11_bindings::{
    bindings::{
        XCB_CW_EVENT_MASK, XCB_EVENT_MASK_SUBSTRUCTURE_NOTIFY, XCB_EVENT_MASK_SUBSTRUCTURE_REDIRECT,
    },
    connection::{self, Connection},
    epoll::Epoll,
    inotify::Inotify,
};

use crate::{
    bar_message::{Message, UnixClients},
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

    let home_dir = std::env::var("HOME").unwrap();

    let pid = std::process::id();
    trace!("PID: {}", pid);

    let mut config_filepath = "config.toml".to_owned();
    let mut config = Config::new(config_filepath.as_str())
        .or_else(|_| {
            config_filepath = format!("{}/.config/x11_wm_rust/config.toml", home_dir);
            Config::new(config_filepath.as_str())
        })
        .or_else(|err| {
            config_filepath = "".to_owned();
            error!("Failed init config from file, error: {:?}", err);
            warn!("Initializing config from default values.");
            Ok::<Config, ConfigErrors>(Config::default())
        })
        .unwrap();
    let mut inotify = Inotify::new().expect("failed to create inotify instance");
    if !config_filepath.is_empty() {
        inotify
            .add_watch(config_filepath.as_str())
            .expect("failed to add config file to watch list");
    }

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
    trace!("monitor dimensions: {:?}", monitor.rect);

    let mut keybindings = keybindings_from_config(&config);
    keybindings_grab(&keybindings, &conn);
    // trace!("keybindings: {:#?}", keybindings);

    conn.change_cursor("left_ptr");
    conn.flush();

    const SERVER_SOCKET_PATH_STR: &str = "/tmp/x11_wm_rust.socket";
    let server_socket_path = Path::new(SERVER_SOCKET_PATH_STR);
    if server_socket_path.exists() {
        let _ = std::fs::remove_file(server_socket_path);
    }
    let unix_listener =
        UnixListener::bind(SERVER_SOCKET_PATH_STR).expect("failed to create unix listener");
    epoll
        .add_watch(unix_listener.as_raw_fd())
        .expect("failed to add unix listener to epoll watch list");
    let mut unix_clients = UnixClients::new();

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
    if config.wallpapers_command.is_some() && config.wallpapers_path.is_some() {
        execute_command_from_str_wait(
            format!(
                "{} {}",
                config.wallpapers_command.as_ref().unwrap(),
                config.wallpapers_path.as_ref().unwrap()
            )
            .as_str(),
        );
    }

    let mut keyboard_layout_name_current = "".to_string();
    loop {
        let events = epoll
            .wait()
            .expect("epoll failed while waiting for new events");
        for event in events.clone() {
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
                                    &mut unix_clients,
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
                                time,
                            } => {
                                monitor.handle_button_press(
                                    x, y, window, state, detail, &conn, &config, time,
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
                                    keyboard_layout_name_current = keyboard_layout_name.clone();
                                    unix_clients
                                        .notify_all(Message::KeyboardLayout(keyboard_layout_name));
                                }
                            }
                            connection::XcbEvents::RandrScreenChange { width, height } => {
                                trace!("RandrScreenChange: width: {}, heigth: {}", width, height);
                                monitor.update_with_new_dimensions(width, height, &conn, &config);

                                if config.wallpapers_command.is_some()
                                    && config.wallpapers_path.is_some()
                                {
                                    execute_command_from_str_wait(
                                        format!(
                                            "{} {}",
                                            config.wallpapers_command.as_ref().unwrap(),
                                            config.wallpapers_path.as_ref().unwrap()
                                        )
                                        .as_str(),
                                    );
                                }
                            }
                            connection::XcbEvents::DestroyNotify { window } => {
                                monitor.handle_destroy_notify(window, &conn, &config);
                            }
                            connection::XcbEvents::UnmapNotify { window } => {
                                trace!("unmap notify for window: {}", window);
                            }
                        },
                        Err(error) => warn!("Error event: {:?}", error),
                    };
                }
                monitor.check_deleted(&conn);
                conn.flush();
            } else if event.u64 == inotify.fd as u64 {
                let mut stop_poll = false;
                loop {
                    if stop_poll {
                        break;
                    }
                    match inotify.read_event() {
                        Ok(event) => match event {
                            x11_bindings::inotify::InotifyEvent::Modify => {
                                trace!("config file was updated, reloading...");
                                match Config::new(config_filepath.as_str()) {
                                    Ok(new_config) => {
                                        trace!(
                                            "updated config parsed successfully: {:#?}",
                                            new_config
                                        );

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

                                        if config.wallpapers_command
                                            != new_config.wallpapers_command
                                            || config.wallpapers_path != new_config.wallpapers_path
                                        {
                                            if new_config.wallpapers_command.is_some()
                                                && new_config.wallpapers_path.is_some()
                                            {
                                                execute_command_from_str_wait(
                                                    format!(
                                                        "{} {}",
                                                        new_config
                                                            .wallpapers_command
                                                            .as_ref()
                                                            .unwrap(),
                                                        new_config
                                                            .wallpapers_path
                                                            .as_ref()
                                                            .unwrap()
                                                    )
                                                    .as_str(),
                                                );
                                            }
                                        }

                                        let new_keybindings = keybindings_from_config(&config);
                                        if keybindings != new_keybindings {
                                            keybindings_ungrab(&keybindings, &conn);
                                            keybindings_grab(&new_keybindings, &conn);
                                            keybindings = new_keybindings;
                                        }

                                        monitor.remap_windows_with_upd_config(
                                            &conn,
                                            &config,
                                            &new_config,
                                        );
                                        config = new_config;

                                        conn.flush();
                                    }
                                    Err(err) => {
                                        error!(
                                            "Failed to reload config from file, error: {:?}",
                                            err
                                        )
                                    }
                                }
                            }
                            x11_bindings::inotify::InotifyEvent::Eof => {
                                stop_poll = true;
                            }
                            _ => {
                                trace!("other inotify event: {:?}", event);
                            }
                        },
                        Err(err) => warn!("inotify read error: {:?}", err),
                    }
                }
            } else if event.u64 == unix_listener.as_raw_fd() as u64 {
                match unix_listener.accept() {
                    Ok((client, _)) => {
                        if let Err(err) = epoll.add_watch(client.as_raw_fd()) {
                            warn!("failed to add unix client to epoll watch list: {}", err);
                        } else {
                            unix_clients.add_client(client);
                        }
                    }
                    Err(err) => warn!("unix listener failed to accept connection: {}", err),
                }
            } else {
                if let Some(client_stream) = unix_clients.find_client_by_fd(event.u64) {
                    if let Some(message) = Message::read_from_unix_stream(client_stream) {
                        trace!("message from unix stream client: {:?}", message);
                        match message {
                            Message::RequestClientInit => {
                                if !keyboard_layout_name_current.is_empty() {
                                    let message =
                                        Message::KeyboardLayout(&keyboard_layout_name_current);
                                    if let Err(err) = client_stream.write_all(&message.as_bytes()) {
                                        warn!(
                                            "failed to write keyboard layout name to client on RequestClientInit: {}",
                                            err
                                        );
                                    }
                                    trace!(
                                        "keyboard layout name sent to unix stream client: {:?}",
                                        message
                                    );
                                }

                                let workspaces_ids = monitor
                                    .workspaces
                                    .iter()
                                    .map(|workspace| workspace.id)
                                    .collect::<Vec<_>>();
                                let message = Message::WorkspaceList(workspaces_ids);
                                if let Err(err) = client_stream.write_all(&message.as_bytes()) {
                                    warn!(
                                        "failed to write workspaces ids to client on RequestClientInit: {}",
                                        err
                                    );
                                }
                                trace!("workspaces ids sent to unix stream client: {:?}", message);

                                if let Some(focused_workspace_id) =
                                    monitor.get_focused_workspace_id()
                                {
                                    let message = Message::WorkspaceActive(focused_workspace_id);
                                    if let Err(err) = client_stream.write_all(&message.as_bytes()) {
                                        warn!(
                                            "failed to write focused workspace id to client on RequestClientInit: {}",
                                            err
                                        );
                                    }
                                    trace!(
                                        "focused workspace id sent to unix stream client: {:?}",
                                        message
                                    );
                                }
                            }
                            _ => {}
                        }
                    } else {
                        warn!("wasn't able to read message sent by client");
                    }
                } else {
                    warn!("received message from unknown client");
                }
            }
        }
    }
}

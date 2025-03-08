#![feature(let_chains)]

use std::{
    process, thread,
    time::{Duration, Instant},
};

use config::{Config, ConfigErrors};
use env_logger::Env;
use keybindings::{
    execute_command_from_str, handle_key_press, keybindings_from_config, keybindings_grab,
};
use log::{error, info, trace, warn};
use monitor::Monitor;
use x11_bindings::{
    bindings::{
        XCB_CW_EVENT_MASK, XCB_EVENT_MASK_POINTER_MOTION, XCB_EVENT_MASK_SUBSTRUCTURE_NOTIFY,
        XCB_EVENT_MASK_SUBSTRUCTURE_REDIRECT,
    },
    connection::{self, Connection},
};

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

    let pid = process::id();
    info!("PID: {}", pid);

    let config = Config::new("config.toml")
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
        XCB_EVENT_MASK_SUBSTRUCTURE_REDIRECT
            | XCB_EVENT_MASK_SUBSTRUCTURE_NOTIFY
            | XCB_EVENT_MASK_POINTER_MOTION,
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

    let keybindings = keybindings_from_config(&config);
    keybindings_grab(&keybindings, &conn);
    // trace!("keybindings: {:#?}", keybindings);

    conn.grab_pointer(XCB_EVENT_MASK_POINTER_MOTION);
    if let Some(cursor_filepath) = &config.custom_cursor_filepath {
        if let Err(err) = conn.set_cursor_filename(cursor_filepath.as_str()) {
            error!(
                "failed to set cursor from filename: {}, error: {:?}",
                cursor_filepath, err
            );
            trace!("falling back to default left_ptr cursor to the root window");
            conn.change_cursor("left_ptr");
        }
    } else {
        conn.change_cursor("left_ptr");
    }

    conn.flush();

    let frame_duration = Duration::from_nanos(16_666_667);
    loop {
        let start_frame_time = Instant::now();

        if let Some(event_res) = conn.poll_for_event() {
            match event_res {
                Ok(event) => match event {
                    connection::XcbEvents::KeyPress { modifier, keycode } => handle_key_press(
                        &keybindings,
                        &conn,
                        &config,
                        &mut monitor,
                        modifier,
                        keycode,
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
                },
                Err(error) => warn!("Error event: {:?}", error),
            }
        }

        monitor.check_deleted(&conn);

        let end_frame_time = Instant::now();
        let delta_duration = end_frame_time - start_frame_time;
        if delta_duration < frame_duration {
            thread::sleep(frame_duration - delta_duration);
        }
    }
}

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

    let keybindings = keybindings_from_config(&config);
    keybindings_grab(&keybindings, &conn);
    // trace!("keybindings: {:#?}", keybindings);

    conn.change_cursor("left_ptr");
    conn.grab_button(MouseButton::Left);

    conn.flush();

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
                    connection::XcbEvents::ButtonPress { x, y, window, state, detail } => {
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
                },
                Err(error) => warn!("Error event: {:?}", error),
            };
        }
        monitor.check_deleted(&conn);
    }
}

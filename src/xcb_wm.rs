use std::process::{Command, Stdio};

use crate::{
    config::Config,
    ewmh,
    icccm::Icccm,
    keybindings::KeyEventsHandler,
    monitor::{Monitor, Rect},
    window::{Window, WindowType},
};
use log::{debug, error, info};
use xcb::{
    randr,
    x::{self, ChangeWindowAttributes, Cw, EventMask},
    Connection, Xid,
};

pub struct XcbWindowManager {
    conn: xcb::Connection,
    // screen_num: i32,
    root_window: x::Window,
    meta_window: x::Window,
    ewmh: ewmh::Ewmh,
    key_events_handler: KeyEventsHandler,
    monitors: Vec<Monitor>,
    focused_monitor: Option<usize>,
}

impl XcbWindowManager {
    pub fn new(window_manager_name: &str, config: &Config) -> Self {
        let (conn, screen_num) = Connection::connect_with_xlib_display_and_extensions(
            &[xcb::Extension::RandR, xcb::Extension::Test],
            &[],
        )
        .expect("Established XCB connection.");

        let root_window = conn
            .get_setup()
            .roots()
            .nth(screen_num as usize)
            .unwrap()
            .root();

        let ewmh = ewmh::Ewmh::new(&conn, root_window);
        let key_events_handler = KeyEventsHandler::new(&conn, root_window, config);

        // let randr_ext = randr::get_extension_data(&conn).unwrap();
        conn.send_request(&randr::SelectInput {
            window: root_window,
            enable: randr::NotifyMask::SCREEN_CHANGE,
        });
        let cookie = conn.send_request(&randr::GetMonitors {
            window: root_window,
            get_active: true,
        });
        let mut monitors = Vec::new();
        let monitors_reply = conn.wait_for_reply(cookie).unwrap();
        for monitor_info in monitors_reply.monitors() {
            // debug!("monitor_info: {:#?}", monitor_info);
            let rect = Rect {
                x: monitor_info.x(),
                y: monitor_info.y(),
                width: monitor_info.width(),
                height: monitor_info.height(),
            };
            let monitor = Monitor::new(root_window, rect, config, &conn);
            monitors.push(monitor);
        }
        let focused_monitor: Option<usize> = if monitors.is_empty() { None } else { Some(0) };

        let mut result = Self {
            conn,
            // screen_num,
            root_window,
            meta_window: x::WINDOW_NONE,
            ewmh,
            key_events_handler,
            monitors,
            focused_monitor,
        };
        result.subscribe_to_wm_events();
        result.meta_window = result.create_meta_window(window_manager_name);
        result
            .ewmh
            .update_current_desktop(&result.conn, result.root_window, 0);

        result.conn.flush().unwrap();

        result
    }

    pub fn handle_event(&mut self, config: &Config) -> bool {
        match self.conn.wait_for_event() {
            Err(xcb::Error::Connection(err)) => {
                error!("Unexpected I/O error: {err}");
                return false;
            }
            Err(xcb::Error::Protocol(err)) => {
                error!("Unexpected protocol error: {err}");
                return false;
            }
            Ok(event) => match event {
                xcb::Event::X(event) => match event {
                    x::Event::KeyPress(event) => {
                        self.key_events_handler.handle_key_press(event);
                    }
                    // x::Event::KeyRelease(_) => todo!(),
                    // x::Event::ButtonPress(_) => todo!(),
                    // x::Event::ButtonRelease(_) => todo!(),
                    // x::Event::MotionNotify(_) => todo!(),
                    // x::Event::EnterNotify(_) => todo!(),
                    // x::Event::LeaveNotify(_) => todo!(),
                    // x::Event::FocusIn(_) => todo!(),
                    // x::Event::FocusOut(_) => todo!(),
                    // x::Event::KeymapNotify(_) => todo!(),
                    // x::Event::Expose(_) => todo!(),
                    // x::Event::GraphicsExposure(_) => todo!(),
                    // x::Event::NoExposure(_) => todo!(),
                    // x::Event::VisibilityNotify(_) => todo!(),
                    // x::Event::CreateNotify(_) => todo!(),
                    // x::Event::DestroyNotify(_) => todo!(),
                    // x::Event::UnmapNotify(_) => todo!(),
                    // x::Event::MapNotify(_) => todo!(),
                    x::Event::MapRequest(event) => {
                        self.handle_map_request(event, config);
                    }
                    // x::Event::ReparentNotify(_) => todo!(),
                    // x::Event::ConfigureNotify(_) => todo!(),
                    // x::Event::ConfigureRequest(_) => todo!(),
                    // x::Event::GravityNotify(_) => todo!(),
                    // x::Event::ResizeRequest(_) => todo!(),
                    // x::Event::CirculateNotify(_) => todo!(),
                    // x::Event::CirculateRequest(_) => todo!(),
                    // x::Event::PropertyNotify(_) => todo!(),
                    // x::Event::SelectionClear(_) => todo!(),
                    // x::Event::SelectionRequest(_) => todo!(),
                    // x::Event::SelectionNotify(_) => todo!(),
                    // x::Event::ColormapNotify(_) => todo!(),
                    // x::Event::ClientMessage(_) => todo!(),
                    // x::Event::MappingNotify(_) => todo!(),
                    _ => {}
                },
                // xcb::Event::Damage(_) => todo!(),
                // xcb::Event::Dri2(_) => todo!(),
                // xcb::Event::Glx(_) => todo!(),
                // xcb::Event::Present(_) => todo!(),
                xcb::Event::RandR(event) => match event {
                    randr::Event::ScreenChangeNotify(event) => {
                        debug!("Randr screen change event: {:#?}", event);
                        for (index, monitor) in self.monitors.iter().enumerate() {
                            if monitor.root_window == event.root() {
                                self.focused_monitor = Some(index);
                                break;
                            }
                        }
                        if self.focused_monitor.is_some() {
                            self.monitors
                                .get_mut(self.focused_monitor.unwrap())
                                .unwrap()
                                .handle_screen_change(event.width(), event.height());
                        }
                    }
                    randr::Event::Notify(event) => {
                        debug!("Randr notify event: {:#?}", event);
                    }
                },
                // xcb::Event::ScreenSaver(_) => todo!(),
                // xcb::Event::Shape(_) => todo!(),
                // xcb::Event::Shm(_) => todo!(),
                // xcb::Event::Sync(_) => todo!(),
                // xcb::Event::XFixes(_) => todo!(),
                // xcb::Event::Input(_) => todo!(),
                // xcb::Event::Xkb(_) => todo!(),
                // xcb::Event::XPrint(_) => todo!(),
                // xcb::Event::Xv(_) => todo!(),
                // xcb::Event::Unknown(_) => todo!(),
                _ => {}
            },
        };

        return true;
    }

    fn handle_map_request(&mut self, event: x::MapRequestEvent, config: &Config) {
        info!("MapRequest - {}", event.window().resource_id());
        let xwindow = event.window();

        let window_attrs_cookie = self
            .conn
            .send_request(&x::GetWindowAttributes { window: xwindow });
        match self.conn.wait_for_reply(window_attrs_cookie) {
            Ok(window_attrs) => {
                if window_attrs.override_redirect() {
                    info!("Option override-redirect is set, ignoring the window");
                    return;
                }
            }
            Err(err) => {
                error!("Failed to get window attributes: {:?}", err);
                return;
            }
        };

        let window_type = self.ewmh.get_window_type(&self.conn, xwindow);
        debug!("Window type: {:?}", window_type);
        // let size_hints_cookie = self.conn.send_request(&x::GetProperty {
        //     delete: false,
        //     window: xwindow,
        //     property: x::ATOM_WM_SIZE_HINTS,
        //     r#type: x::ATOM_ANY,
        //     long_offset: 0,
        //     long_length: 1024,
        // });
        // match self.conn.wait_for_reply(size_hints_cookie) {
        //     Ok(size_hints) => {
        //         debug!("size hints: {:#?}", size_hints);
        //     }
        //     Err(err) => {
        //         error!("Failed to get size hints: {:?}", err);
        //     }
        // };
        let monitor = self
            .monitors
            .get_mut(self.focused_monitor.unwrap())
            .unwrap();
        let mut window = match window_type {
            WindowType::Tiling => {
                let rect = monitor.make_rect_for_tiling_window(&self.conn, config);
                let window = Window::new(rect, xwindow, window_type, config.window.border.size);
                window
            }
            WindowType::Floating => {
                let rect = monitor.make_rect_for_tiling_window(&self.conn, config);
                let window = Window::new(rect, xwindow, window_type, config.window.border.size);
                window
            }
            WindowType::Docking => {
                let rect = monitor.make_rect_for_tiling_window(&self.conn, config);
                let window = Window::new(rect, xwindow, window_type, config.window.border.size);
                window
            }
        };
        window.rect.height -= 2 * config.window.border.size as u16;
        window.rect.width -= 2 * config.window.border.size as u16;
        window.configure(&self.conn);
        window.subscribe_to_wm_events(&self.conn);
        window.show(&self.conn);
        monitor.set_focused(&window, &self.conn, config);

        self.conn.flush().unwrap();
    }

    pub fn execute_startup_commands(&self, config: &Config) {
        for command in &config.startup_commands {
            match Command::new(command.clone())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(_) => info!("Successfully executed command: '{command}'"),
                Err(error) => error!("Failed to executed command: '{command}', error: '{error}'"),
            }
        }
    }

    fn subscribe_to_wm_events(&self) {
        self.conn.send_request(&ChangeWindowAttributes {
            window: self.root_window,
            value_list: &[Cw::EventMask(
                EventMask::SUBSTRUCTURE_REDIRECT
                    | EventMask::SUBSTRUCTURE_NOTIFY
                    | EventMask::POINTER_MOTION,
            )],
        });
    }

    fn create_meta_window(&self, window_manager_name: &str) -> x::Window {
        let meta_window: x::Window = self.conn.generate_id();
        self.conn.send_request(&x::CreateWindow {
            depth: x::COPY_FROM_PARENT as u8,
            wid: meta_window,
            parent: self.root_window,
            x: -1,
            y: -1,
            width: 1,
            height: 1,
            border_width: 0,
            class: x::WindowClass::InputOutput,
            visual: x::COPY_FROM_PARENT,
            value_list: &[],
        });

        Icccm::set_wm_class(&self.conn, meta_window, window_manager_name);

        self.conn.send_request(&x::GrabPointer {
            owner_events: true,
            grab_window: self.root_window,
            event_mask: x::EventMask::POINTER_MOTION,
            pointer_mode: x::GrabMode::Async,
            keyboard_mode: x::GrabMode::Async,
            confine_to: x::WINDOW_NONE,
            cursor: x::CURSOR_NONE,
            time: x::CURRENT_TIME,
        });

        meta_window
    }
}

impl Drop for XcbWindowManager {
    fn drop(&mut self) {
        self.conn.send_request(&x::UngrabPointer {
            time: x::CURRENT_TIME,
        });
        self.conn.send_request(&x::DestroyWindow {
            window: self.meta_window,
        });
    }
}

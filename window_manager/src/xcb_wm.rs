use std::process::{Command, Stdio};

use crate::{
    config::Config,
    ewmh,
    icccm::Icccm,
    keybindings::KeyEventsHandler,
    monitor::{Monitor, Rect},
    window::{Window, WindowType},
    workspace,
};
use image::{imageops::FilterType, ColorType};
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
    root_depth: u8,
    root_visual: x::Visualid,
    root_black_pixel: u32,
    root_white_pixel: u32,
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

        let screen = conn.get_setup().roots().nth(screen_num as usize).unwrap();
        let root_window = screen.root();
        let root_depth = screen.root_depth();
        let root_visual = screen.root_visual();
        let root_black_pixel = screen.black_pixel();
        let root_white_pixel = screen.white_pixel();

        let ewmh = ewmh::Ewmh::new(&conn, root_window);
        let key_events_handler = KeyEventsHandler::new(&conn, root_window, config);

        // result.set_cursor();
        let cursor: x::Cursor = conn.generate_id();
        conn.send_request(&x::GrabPointer {
            owner_events: false, // get all pointer events specified by the following mask
            grab_window: root_window, // grab the root window
            event_mask: x::EventMask::NO_EVENT, // which event to let through
            pointer_mode: x::GrabMode::Async, // pointer events should continue as normal
            keyboard_mode: x::GrabMode::Async, // pointer events should continue as normal
            confine_to: x::Window::none(), // in which window should the cursor stay
            cursor,              // we change the cursor
            time: x::CURRENT_TIME,
        });

        conn.flush().unwrap();

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
            root_window,
            root_depth,
            root_visual,
            root_black_pixel,
            root_white_pixel,
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
        match self.conn.poll_for_event() {
            Err(xcb::Error::Connection(err)) => {
                error!("Unexpected I/O error: {err:?}");
                return false;
            }
            Err(xcb::Error::Protocol(err)) => {
                error!("Unexpected protocol error: {err:?}");
                // return false;
            }
            Ok(event) => match event {
                Some(xcb::Event::X(event)) => match event {
                    x::Event::KeyPress(event) => self.key_events_handler.handle_key_press(event),
                    // x::Event::KeyRelease(_) => todo!(),
                    // x::Event::ButtonPress(_) => todo!(),
                    // x::Event::ButtonRelease(_) => todo!(),
                    x::Event::MotionNotify(event) => self.handle_motion_event(event),
                    x::Event::EnterNotify(event) => self.handle_enter_notify_event(event, config),
                    // x::Event::LeaveNotify(_) => todo!(),
                    // x::Event::FocusIn(event) => self.handle_focus_in_event(event, config),
                    // x::Event::FocusOut(event) => self.handle_focus_out_event(event, config),
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
                Some(xcb::Event::RandR(event)) => match event {
                    randr::Event::ScreenChangeNotify(event) => {
                        // debug!("Randr screen change event: {:#?}", event);
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
        let window_id = event.window();

        let window_attrs_cookie = self
            .conn
            .send_request(&x::GetWindowAttributes { window: window_id });
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

        let window_type = self.ewmh.get_window_type(&self.conn, window_id);
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
        match window_type {
            WindowType::Tiling => {
                monitor.add_tiling_window_to_focused_workspace(&self.conn, config, window_id);
            }
            WindowType::Floating => {}
            WindowType::Docking => {
                let desktop_index = self.ewmh.get_desktop_index(&self.conn, window_id);
                if desktop_index.is_none() || desktop_index.unwrap() == 0xFFFFFFFF {
                    debug!("docking window with global desktop index");
                } else {
                    debug!("docking window with specific desktop index");
                    monitor.set_workspace_focused(
                        desktop_index.unwrap() as u16,
                        config,
                        &self.conn,
                    );
                }
            }
        };

        self.conn.flush().unwrap();
    }

    fn handle_focus_in_event(&self, event: x::FocusInEvent, config: &Config) {
        debug!("FocusIn: {}", event.event().resource_id());
        if event.event() != self.root_window {
            let monitor = self.monitors.get(self.focused_monitor.unwrap()).unwrap();
            if let Some(window) = monitor
                .get_focused_workspace()
                .unwrap()
                .get_window(event.event())
            {
                window.change_border_color(
                    &self.conn,
                    config.window.border.color_active_u32.unwrap(),
                );
                self.conn.flush().unwrap();
            }
        }
    }

    fn handle_focus_out_event(&self, event: x::FocusOutEvent, config: &Config) {
        debug!("FocusOut: {}", event.event().resource_id());
        if event.event() != self.root_window {
            let monitor = self.monitors.get(self.focused_monitor.unwrap()).unwrap();
            if let Some(window) = monitor
                .get_focused_workspace()
                .unwrap()
                .get_window(event.event())
            {
                window.change_border_color(
                    &self.conn,
                    config.window.border.color_inactive_u32.unwrap(),
                );
                self.conn.flush().unwrap();
            }
        }
    }

    fn handle_enter_notify_event(&mut self, event: x::EnterNotifyEvent, config: &Config) {
        debug!("Enter Notify: {}", event.event().resource_id());
        if event.event() != self.root_window {
            let monitor = self
                .monitors
                .get_mut(self.focused_monitor.unwrap())
                .unwrap();
            let workspace = monitor.get_focused_workspace_mut().unwrap();
            workspace.set_window_focused_by_id(event.event(), &self.conn, false, config);
            self.conn.flush().unwrap();
        }
    }

    // fn handle_leave_notify_event(&self, event: x::LeaveNotifyEvent) {
    //     debug!("Leave Notify: {:?}", event.event());
    // }

    #[inline]
    fn handle_motion_event(&mut self, event: x::MotionNotifyEvent) {
        let monitor = self
            .monitors
            .get_mut(self.focused_monitor.unwrap())
            .unwrap();
        let workspace = monitor.get_focused_workspace_mut().unwrap();
        if workspace.is_keyboard_focused() {
            // debug!("event_x: {}, event_y: {}", event.event_x(), event.event_y());
            let focused_window = workspace.get_focused_window();
            let window = workspace.get_window_under_cursor(event.event_x(), event.event_y());
            if focused_window == window {
                workspace.reset_keyboard_focused_flag();
            }
        }
    }

    pub fn execute_startup_commands(&self, config: &Config) {
        for command in &config.startup_commands {
            let segments = command.split(' ').collect::<Vec<_>>();
            match Command::new(segments.first().unwrap())
                .args(segments[1..].iter())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(_) => info!("Successfully executed command: '{command}'"),
                Err(error) => error!("Failed to execute command: '{command}', error: '{error}'"),
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

    fn set_cursor(&self) {
        let img = image::open("cursor_image1.jpg").unwrap();
        debug!("width: {}, height: {}", img.width(), img.height());
        let img = img
            .crop_imm(105, 4, 492, 492)
            .resize(32, 32, FilterType::Nearest);
        let width = img.width() as u16;
        let height = img.height() as u16;
        debug!("width: {}, height: {}", width, height);
        // let bytes = img.grayscale().to_rgb8().into_raw();
        let bytes = img.to_rgb8().into_raw();
        let color_type = img.color();
        let depth = match color_type {
            ColorType::L8 | ColorType::La8 | ColorType::Rgb8 | ColorType::Rgba8 => 8,
            ColorType::L16 | ColorType::La16 | ColorType::Rgb16 | ColorType::Rgba16 => 16,
            _ => 0, // Handle unsupported or unknown color types
        };
        let depth = depth * color_type.channel_count() as u8;
        debug!("depth: {}", depth);

        // let mask_bitmap: [u8; 8] = [
        //     0b00011000, 0b00111100, 0b01111110, 0b11111111, 0b01111110, 0b00111100, 0b00011000,
        //     0b00000000,
        // ];
        // let mut binary_bitmap = String::new();
        // for b in mask_bitmap {
        //     binary_bitmap.push_str(format!("0b{:08b}", b).as_str());
        //     binary_bitmap.push_str(", ");
        // }
        // debug!("bitmap: {}", binary_bitmap);

        let drawable = x::Drawable::Window(self.root_window);

        let cursor_pixmap: x::Pixmap = self.conn.generate_id();
        self.conn.send_request(&x::CreatePixmap {
            depth,
            pid: cursor_pixmap,
            drawable,
            width,
            height,
        });

        let mask_pixmap: x::Pixmap = self.conn.generate_id();
        self.conn.send_request(&x::CreatePixmap {
            depth,
            pid: mask_pixmap,
            drawable,
            width,
            height,
        });

        let gc: x::Gcontext = self.conn.generate_id();
        self.conn.send_request(&x::CreateGc {
            cid: gc,
            // drawable: x::Drawable::Pixmap(cursor_pixmap),
            drawable,
            value_list: &[
                x::Gc::Foreground(self.root_black_pixel),
                x::Gc::Background(self.root_white_pixel),
            ],
        });

        self.conn.send_request(&x::PutImage {
            format: x::ImageFormat::XyPixmap,
            drawable: x::Drawable::Pixmap(cursor_pixmap),
            // drawable,
            gc,
            width,
            height,
            dst_x: 0,
            dst_y: 0,
            left_pad: 0,
            depth,
            data: &bytes,
        });

        self.conn.send_request(&x::PutImage {
            format: x::ImageFormat::XyPixmap,
            drawable: x::Drawable::Pixmap(mask_pixmap),
            // drawable,
            gc,
            width,
            height,
            dst_x: 0,
            dst_y: 0,
            left_pad: 0,
            depth,
            data: &bytes,
        });

        let cursor: x::Cursor = self.conn.generate_id();
        self.conn.send_request(&x::CreateCursor {
            cid: cursor,
            source: cursor_pixmap,
            mask: cursor_pixmap,
            fore_red: 0x0000,
            fore_green: 0x0000,
            fore_blue: 0x0000,
            back_red: 0x0000,
            back_green: 0x0000,
            back_blue: 0x0000,
            x: width / 2,
            y: 0,
        });
        self.conn.send_request(&x::ChangeWindowAttributes {
            window: self.root_window,
            value_list: &[x::Cw::Cursor(cursor)],
        });
    }

    pub fn handle_shift_focus_left(&mut self, config: &Config) {
        debug!("handle shift focus left.");
        let monitor = self
            .monitors
            .get_mut(self.focused_monitor.unwrap())
            .unwrap();
        let workspace = monitor.get_focused_workspace_mut().unwrap();
        workspace.shift_focus_left(&self.conn, config);
        self.conn.flush().unwrap();
    }

    pub fn handle_shift_focus_right(&mut self, config: &Config) {
        debug!("handle shift focus right.");
        let monitor = self
            .monitors
            .get_mut(self.focused_monitor.unwrap())
            .unwrap();
        let workspace = monitor.get_focused_workspace_mut().unwrap();
        workspace.shift_focus_right(&self.conn, config);
        self.conn.flush().unwrap();
    }

    pub fn handle_shift_focus_up(&mut self, config: &Config) {
        debug!("handle shift focus up.");
        let monitor = self
            .monitors
            .get_mut(self.focused_monitor.unwrap())
            .unwrap();
        let workspace = monitor.get_focused_workspace_mut().unwrap();
        match workspace.get_focused_window_type() {
            Some(WindowType::Tiling) => {
                if let Some(new_workspace_id) = monitor.get_upper_workspace_id() {
                    monitor.set_workspace_focused(new_workspace_id, config, &self.conn);
                    self.conn.flush().unwrap();
                }
            }
            _ => {}
        }
    }

    pub fn handle_shift_focus_down(&mut self, config: &Config) {
        debug!("handle shift focus down.");
        let monitor = self
            .monitors
            .get_mut(self.focused_monitor.unwrap())
            .unwrap();
        let workspace = monitor.get_focused_workspace_mut().unwrap();
        match workspace.get_focused_window_type() {
            Some(WindowType::Tiling) => {
                if let Some(new_workspace_id) = monitor.get_lower_workspace_id() {
                    monitor.set_workspace_focused(new_workspace_id, config, &self.conn);
                    self.conn.flush().unwrap();
                }
            }
            _ => {}
        }
    }

    pub fn handle_window_move_left(&mut self, config: &Config) {
        debug!("handle window move left");
        let monitor = self
            .monitors
            .get_mut(self.focused_monitor.unwrap())
            .unwrap();
        let workspace = monitor.get_focused_workspace_mut().unwrap();
        workspace.move_window_left(&self.conn, config);
        self.conn.flush().unwrap();
    }

    pub fn handle_window_move_right(&mut self, config: &Config) {
        debug!("handle window move right");
        let monitor = self
            .monitors
            .get_mut(self.focused_monitor.unwrap())
            .unwrap();
        let workspace = monitor.get_focused_workspace_mut().unwrap();
        workspace.move_window_right(&self.conn, config);
        self.conn.flush().unwrap();
    }

    pub fn handle_change_workspace_id(&mut self, new_workspace_id: u16, config: &Config) {
        debug!("change workspace to {}", new_workspace_id);
        let monitor = self
            .monitors
            .get_mut(self.focused_monitor.unwrap())
            .unwrap();
        monitor.set_workspace_focused(new_workspace_id, config, &self.conn);
        self.conn.flush().unwrap();
    }

    pub fn handle_selected_window_grows_width(&mut self, pixels: u16) {
        debug!("grow window width");
        assert!(pixels != 0);
        let monitor = self
            .monitors
            .get_mut(self.focused_monitor.unwrap())
            .unwrap();
        let workspace = monitor.get_focused_workspace_mut().unwrap();
        workspace.grow_width_selected_window(&self.conn, pixels);
        self.conn.flush().unwrap();
    }

    pub fn handle_selected_window_shrink_width(&mut self, pixels: u16, config: &Config) {
        debug!("shrink window width");
        assert!(pixels != 0);
        let monitor = self
            .monitors
            .get_mut(self.focused_monitor.unwrap())
            .unwrap();
        let workspace = monitor.get_focused_workspace_mut().unwrap();
        workspace.shrink_width_selected_window(&self.conn, pixels, config);
        self.conn.flush().unwrap();
    }

    fn move_window_to_workspace_id(&mut self, new_workspace_id: u16, config: &Config) {
        assert!(new_workspace_id != 0);
        let monitor = self
            .monitors
            .get_mut(self.focused_monitor.unwrap())
            .unwrap();
        let monitor_height = monitor.rect.height;
        let workspace = monitor.get_focused_workspace_mut().unwrap();
        // debug!("workspace: {:#?}", workspace);
        if let Some(window) = workspace.remove_selected_window(&self.conn, config, monitor_height) {
            if let Some(new_workspace) = monitor.get_workspace_by_id_mut(new_workspace_id) {
                new_workspace.add_existing_tiling_window(
                    &self.conn,
                    window,
                    config,
                    monitor_height,
                );
                if config.switch_workspace_on_window_workspace_change {
                    new_workspace.unhide_all_windows(monitor_height, &self.conn);
                }
            } else {
                monitor.add_new_workspace(new_workspace_id, config, &self.conn);
                let new_workspace = monitor.get_workspace_by_id_mut(new_workspace_id).unwrap();
                new_workspace.add_existing_tiling_window(
                    &self.conn,
                    window,
                    config,
                    monitor_height,
                );
                if config.switch_workspace_on_window_workspace_change {
                    new_workspace.unhide_all_windows(monitor_height, &self.conn);
                }
                // debug!("new workspace: {:#?}", new_workspace);
            }
            self.conn.flush().unwrap();
        }
    }

    pub fn handle_workspace_change_for_selected_window(
        &mut self,
        new_workspace_id: u16,
        config: &Config,
    ) {
        debug!("selected window workspace change");
        self.move_window_to_workspace_id(new_workspace_id, config);
    }

    pub fn handle_window_move_up(&mut self, config: &Config) {
        debug!("handle window move up");
        let monitor = self
            .monitors
            .get_mut(self.focused_monitor.unwrap())
            .unwrap();
        let new_workspace_id = if let Some(new_workspace_id) = monitor.get_upper_workspace_id() {
            new_workspace_id
        } else {
            let focused_workspace_id = monitor.get_focused_workspace().unwrap().id;
            if focused_workspace_id > 1 {
                monitor.add_new_workspace(focused_workspace_id - 1, config, &self.conn);
                focused_workspace_id - 1
            } else {
                0
            }
        };
        if new_workspace_id != 0 {
            self.move_window_to_workspace_id(new_workspace_id, config);
        }
    }

    pub fn handle_window_move_down(&mut self, config: &Config) {
        debug!("handle window move down");
        let monitor = self
            .monitors
            .get_mut(self.focused_monitor.unwrap())
            .unwrap();
        let new_workspace_id = if let Some(new_workspace_id) = monitor.get_lower_workspace_id() {
            new_workspace_id
        } else {
            let focused_workspace_id = monitor.get_focused_workspace().unwrap().id;
            if focused_workspace_id + 1 < config.workspaces.count {
                monitor.add_new_workspace(focused_workspace_id + 1, config, &self.conn);
                focused_workspace_id + 1
            } else {
                config.workspaces.count
            }
        };
        if new_workspace_id != config.workspaces.count {
            self.move_window_to_workspace_id(new_workspace_id, config);
        }
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

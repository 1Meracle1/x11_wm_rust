use base::Rect;
use log::trace;
use x11_bindings::bindings::{
    XCB_CW_BORDER_PIXEL, XCB_CW_EVENT_MASK, XCB_EVENT_MASK_FOCUS_CHANGE, XCB_NOTIFY_MODE_GRAB,
    XCB_NOTIFY_MODE_UNGRAB, xcb_notify_mode_t, xcb_window_t,
};

use crate::{
    config::Config,
    connection::{Connection, WindowType},
    keybindings::Direction,
    window::Window,
    workspace::Workspace,
};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Monitor {
    pub rect: Rect,
    pub workspaces: Vec<Workspace>,
    pub docked: Vec<Window>,
    pub focused_workspace_idx: usize,
}

impl Monitor {
    #[allow(dead_code)]
    pub fn new(conn: &Connection) -> Self {
        Self {
            rect: conn.screen_rect(),
            workspaces: vec![Workspace::new(0)],
            docked: Vec::new(),
            focused_workspace_idx: 0,
        }
    }

    #[allow(dead_code)]
    pub fn handle_map_request(&mut self, conn: &Connection, config: &Config, window: xcb_window_t) {
        trace!("map request: {}", window);
        if conn.has_override_redirect(window) {
            trace!("override-redirect enabled, skipping window");
            return;
        }

        let requested_workspace = conn.window_requested_workspace(window);
        if let Some((class_name, instance_name)) = conn.window_class_instance_names(window) {
            trace!(
                "window class name: {}, instance name: {}",
                class_name, instance_name
            );
        }

        let mut should_apply_focus = true;
        let window_type = conn.window_type(window);
        trace!("Window type: {:?}", window_type);
        match window_type {
            WindowType::Normal => {
                let focused_workspace =
                    self.workspaces.get_mut(self.focused_workspace_idx).unwrap();
                focused_workspace.handle_new_normal_window(window, &self.rect, conn, config);
            }
            WindowType::Floating => {
                let focused_workspace =
                    self.workspaces.get_mut(self.focused_workspace_idx).unwrap();
                let base_window_rect = if let Some(rect_hints) = conn.window_rect_hints(window) {
                    trace!("window rect hints: {:?}", rect_hints);
                    rect_hints
                } else {
                    Rect::default()
                };
                focused_workspace.handle_new_floating_window(
                    window,
                    base_window_rect,
                    &self.rect,
                    conn,
                    config,
                );
            }
            WindowType::Docked => {
                if let Some(partial_strut) = conn.window_strut_partial(window, &conn.screen_rect())
                {
                    trace!("window partial strut: {:?}", partial_strut);
                }
                if let Some(workspace) = requested_workspace {
                    trace!("Window requested workspace: {}", workspace);
                    if workspace != self.workspaces[self.focused_workspace_idx].id {
                        should_apply_focus = false;
                    }
                } else {
                    should_apply_focus = false;
                }
                if should_apply_focus {
                    conn.change_window_attrs(
                        window,
                        XCB_CW_BORDER_PIXEL,
                        config.border_color_inactive_int.unwrap(),
                    );
                    conn.change_window_attrs(
                        window,
                        XCB_CW_EVENT_MASK,
                        XCB_EVENT_MASK_FOCUS_CHANGE,
                    );
                }
                conn.map_window(window);
                if should_apply_focus {
                    trace!("apply focus to {}", window);
                    conn.window_set_input_focus(window);
                }
            }
        }

        conn.flush();
    }

    #[allow(dead_code)]
    pub fn handle_focus_in(
        &mut self,
        conn: &Connection,
        config: &Config,
        window: xcb_window_t,
        mode: xcb_notify_mode_t,
    ) {
        if mode == XCB_NOTIFY_MODE_GRAB || mode == XCB_NOTIFY_MODE_UNGRAB {
            // trace!("ignoring focus_in notification due to mode either being GRAB or UNGRAB");
            return;
        }
        if window == conn.root() {
            // trace!("ignoring focus_in notification due to window being root");
            return;
        }
        trace!("focus_in {}", window);
        conn.change_window_attrs(
            window,
            XCB_CW_BORDER_PIXEL,
            config.border_color_active_int.unwrap(),
        );
        conn.flush();
    }

    #[allow(dead_code)]
    pub fn handle_focus_out(
        &mut self,
        conn: &Connection,
        config: &Config,
        window: xcb_window_t,
        mode: xcb_notify_mode_t,
    ) {
        if mode == XCB_NOTIFY_MODE_GRAB || mode == XCB_NOTIFY_MODE_UNGRAB {
            // trace!("ignoring focus_out notification due to mode either being GRAB or UNGRAB");
            return;
        }
        if window == conn.root() {
            // trace!("ignoring focus_out notification due to window being root");
            return;
        }
        trace!("focus_out {}", window);
        conn.change_window_attrs(
            window,
            XCB_CW_BORDER_PIXEL,
            config.border_color_inactive_int.unwrap(),
        );
        conn.flush();
    }

    #[allow(dead_code)]
    pub fn handle_focus_window_change(
        &mut self,
        conn: &Connection,
        config: &Config,
        direction: Direction,
    ) {
        trace!("focus window change: {:?}", direction);
        let focused_workspace = self.workspaces.get_mut(self.focused_workspace_idx).unwrap();
        match direction {
            Direction::Left => {
                focused_workspace.handle_change_focus_window_left(conn, config, &self.rect)
            }
            Direction::Right => {
                focused_workspace.handle_change_focus_window_right(conn, config, &self.rect)
            }
            Direction::Up => todo!(),
            Direction::Down => todo!(),
        };
        conn.flush();
    }

    #[allow(dead_code)]
    pub fn handle_move_window(&mut self, conn: &Connection, config: &Config, direction: Direction) {
        trace!("move window: {:?}", direction);
        let focused_workspace = self.workspaces.get_mut(self.focused_workspace_idx).unwrap();
        match direction {
            Direction::Left => focused_workspace.handle_move_window_left(conn, config, &self.rect),
            Direction::Right => {
                focused_workspace.handle_move_window_right(conn, config, &self.rect)
            }
            Direction::Up => todo!(),
            Direction::Down => todo!(),
        };
        conn.flush();
    }
}

use std::u32;

use base::Rect;
use log::{error, trace};
use x11_bindings::bindings::{
    XCB_CW_BORDER_PIXEL, XCB_NOTIFY_MODE_GRAB, XCB_NOTIFY_MODE_UNGRAB, xcb_notify_mode_t,
    xcb_window_t,
};

use crate::{
    config::Config,
    connection::{Connection, WindowType},
    keybindings::{Dimension, Direction},
    window::DockedWindows,
    workspace::Workspace,
};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Monitor {
    pub rect: Rect,
    pub workspaces: Vec<Workspace>,
    pub docked: DockedWindows,
    pub focused_workspace_idx: usize,
}

impl Monitor {
    pub fn new(conn: &Connection) -> Self {
        Self {
            rect: conn.screen_rect(),
            workspaces: vec![Workspace::new(1)],
            docked: DockedWindows::new(1),
            focused_workspace_idx: 0,
        }
    }

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

        let avail_rect = self
            .rect
            .available_rect_after_adding_rects(self.docked.rects());

        let window_type = conn.window_type(window);
        trace!("Window type: {:?}", window_type);
        match window_type {
            WindowType::Normal => {
                let focused_workspace =
                    self.workspaces.get_mut(self.focused_workspace_idx).unwrap();
                focused_workspace.handle_new_normal_window(window, &avail_rect, conn, config);
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
                    &avail_rect,
                    conn,
                    config,
                );
            }
            WindowType::Docked => {
                if let Some(partial_strut) = conn.window_strut_partial(window, &conn.screen_rect())
                {
                    trace!("window partial strut: {:?}", partial_strut);
                    let workspace_id = requested_workspace.or(Some(u32::MAX)).unwrap();
                    trace!("Window requested workspace: {}", workspace_id);
                    let is_globally_docked = workspace_id > 10;
                    if is_globally_docked {
                        if avail_rect.width != 0 && avail_rect.height != 0 {
                            let magnified_rect = avail_rect.new_rect_magnified(&partial_strut);
                            conn.window_configure(window, &magnified_rect, 0);
                            conn.map_window(window);
                            self.docked.add(window, magnified_rect);
                        } else {
                            error!(
                                "can't add globally docked window {} as there is no more elements that can fit on the screen",
                                window
                            );
                        }
                    } else {
                        let target_workspace_idx = if let Some((idx, _)) = self
                            .workspaces
                            .iter()
                            .enumerate()
                            .find(|(_, w)| w.id == workspace_id)
                        {
                            idx
                        } else {
                            self.workspaces.push(Workspace::new(workspace_id));
                            self.workspaces.len() - 1
                        };
                        let avail_rect = self
                            .rect
                            .available_rect_after_adding_rects(self.docked.rects());
                        self.workspaces
                            .get_mut(target_workspace_idx)
                            .unwrap()
                            .handle_new_docked_window(window, &partial_strut, &avail_rect, conn);
                    }
                } else {
                    error!("for docked window {} no (partial) strut provided", window);
                }
            }
        }

        conn.flush();
    }

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

    pub fn handle_focus_window_change(
        &mut self,
        conn: &Connection,
        config: &Config,
        direction: Direction,
    ) {
        trace!("focus window change: {:?}", direction);
        let avail_rect = self
            .rect
            .available_rect_after_adding_rects(self.docked.rects());
        let focused_workspace = self.workspaces.get_mut(self.focused_workspace_idx).unwrap();
        match direction {
            Direction::Left => {
                focused_workspace.handle_change_focus_window_left(conn, config, &avail_rect)
            }
            Direction::Right => {
                focused_workspace.handle_change_focus_window_right(conn, config, &avail_rect)
            }
            Direction::Up => todo!(),
            Direction::Down => todo!(),
        };
        conn.flush();
    }

    pub fn handle_move_window(&mut self, conn: &Connection, config: &Config, direction: Direction) {
        trace!("move window: {:?}", direction);
        let avail_rect = self
            .rect
            .available_rect_after_adding_rects(self.docked.rects());
        let focused_workspace = self.workspaces.get_mut(self.focused_workspace_idx).unwrap();
        match direction {
            Direction::Left => focused_workspace.handle_move_window_left(conn, config, &avail_rect),
            Direction::Right => {
                focused_workspace.handle_move_window_right(conn, config, &avail_rect)
            }
            Direction::Up => todo!(),
            Direction::Down => todo!(),
        };
        conn.flush();
    }

    pub fn handle_resize_window(
        &mut self,
        conn: &Connection,
        config: &Config,
        dimension: Dimension,
        size_change_pixels: i32,
    ) {
        trace!(
            "resize window: {:?}, pixels: {}",
            dimension, size_change_pixels
        );
        if size_change_pixels == 0 {
            return;
        }
        let avail_rect = self
            .rect
            .available_rect_after_adding_rects(self.docked.rects());
        let focused_workspace = self.workspaces.get_mut(self.focused_workspace_idx).unwrap();
        match dimension {
            Dimension::Horizontal => {
                focused_workspace.handle_resize_window_horizontal(
                    conn,
                    config,
                    &avail_rect,
                    size_change_pixels,
                );
            }
            Dimension::Vertical => todo!(),
        };
        conn.flush();
    }

    pub fn handle_switch_to_workspace(
        &mut self,
        conn: &Connection,
        config: &Config,
        workspace_id: u32,
    ) {
        let focused_workspace_id = self.workspaces.get(self.focused_workspace_idx).unwrap().id;
        trace!(
            "switch to workspace: {}, currently focused workspace id: {}, index: {}",
            workspace_id, focused_workspace_id, self.focused_workspace_idx,
        );
        if workspace_id == focused_workspace_id {
            return;
        }
        let avail_rect = self
            .rect
            .available_rect_after_adding_rects(self.docked.rects());

        let hide_below = workspace_id < focused_workspace_id;
        self.workspaces
            .get_mut(self.focused_workspace_idx)
            .unwrap()
            .hide_all_windows(&avail_rect, conn, config, hide_below);

        let new_focused_workspace_idx = if let Some((idx, _)) = self
            .workspaces
            .iter()
            .enumerate()
            .find(|(_, w)| w.id == workspace_id)
        {
            idx
        } else {
            self.workspaces.push(Workspace::new(workspace_id));
            self.workspaces.len() - 1
        };
        self.focused_workspace_idx = new_focused_workspace_idx;
        self.workspaces
            .get_mut(self.focused_workspace_idx)
            .unwrap()
            .show_all_windows(&avail_rect, conn, config);

        conn.flush();
    }

    pub fn handle_move_focused_window_to_workspace(
        &mut self,
        conn: &Connection,
        config: &Config,
        workspace_id: u32,
    ) {
        let focused_workspace_id = self.workspaces.get(self.focused_workspace_idx).unwrap().id;
        trace!(
            "move focused window to workspace: {}, currently focused workspace id: {}, index: {}",
            workspace_id, focused_workspace_id, self.focused_workspace_idx,
        );
        if workspace_id == focused_workspace_id {
            return;
        }
        let avail_rect = self
            .rect
            .available_rect_after_adding_rects(self.docked.rects());

        if config.switch_to_workspace_on_focused_window_moved {
            let hide_below = workspace_id < focused_workspace_id;
            self.workspaces
                .get_mut(self.focused_workspace_idx)
                .unwrap()
                .hide_all_windows(&avail_rect, conn, config, hide_below);
        }

        let new_focused_workspace_idx = if let Some((idx, _)) = self
            .workspaces
            .iter()
            .enumerate()
            .find(|(_, w)| w.id == workspace_id)
        {
            idx
        } else {
            self.workspaces.push(Workspace::new(workspace_id));
            self.workspaces.len() - 1
        };

        if let Some((window, window_type)) = self
            .workspaces
            .get_mut(self.focused_workspace_idx)
            .unwrap()
            .pop_focused_window()
        {
            match window_type {
                WindowType::Normal => {
                    // self.workspaces
                    //     .get_mut(new_focused_workspace_idx)
                    //     .unwrap()
                    //     .handle_new_normal_window(window, monitor_rect, conn, config);
                    todo!()
                }
                WindowType::Floating => todo!(),
                WindowType::Docked => todo!(),
            }
        }

        if config.switch_to_workspace_on_focused_window_moved {
            self.focused_workspace_idx = new_focused_workspace_idx;
            self.workspaces
                .get_mut(self.focused_workspace_idx)
                .unwrap()
                .show_all_windows(&avail_rect, conn, config);
        }

        conn.flush();
    }
}

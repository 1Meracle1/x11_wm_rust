use std::{time::UNIX_EPOCH, u32};

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
    window::WindowsCollection,
    workspace::Workspace,
};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Monitor {
    rect: Rect,
    workspaces: Vec<Workspace>,
    docked: WindowsCollection,
    focused_workspace_idx: usize,
    to_check_deleted: Vec<(xcb_window_t, u64)>, // window and timestamp when it was requested to be deleted
}

impl Monitor {
    pub fn new(conn: &Connection) -> Self {
        Self {
            rect: conn.screen_rect(),
            workspaces: vec![Workspace::new(1, true)],
            docked: WindowsCollection::new(1),
            focused_workspace_idx: 0,
            to_check_deleted: vec![],
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
            .available_rect_after_adding_rects(self.docked.rect_iter());

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
                            self.docked.add(window, magnified_rect, true);
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
                            self.workspaces.push(Workspace::new(workspace_id, false));
                            self.workspaces.len() - 1
                        };
                        let avail_rect = self
                            .rect
                            .available_rect_after_adding_rects(self.docked.rect_iter());
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
        trace!("focus_in {}", window);
        if mode == XCB_NOTIFY_MODE_GRAB || mode == XCB_NOTIFY_MODE_UNGRAB {
            // trace!("ignoring focus_in notification due to mode either being GRAB or UNGRAB");
            return;
        }
        if window == conn.root() {
            // trace!("ignoring focus_in notification due to window being root");
            return;
        }

        self.workspaces
            .get(self.focused_workspace_idx)
            .unwrap()
            .handle_focus_in(window, conn, config);

        conn.flush();
    }

    pub fn handle_focus_out(
        &mut self,
        conn: &Connection,
        config: &Config,
        window: xcb_window_t,
        mode: xcb_notify_mode_t,
    ) {
        trace!("focus_out {}", window);
        if mode == XCB_NOTIFY_MODE_GRAB || mode == XCB_NOTIFY_MODE_UNGRAB {
            // trace!("ignoring focus_out notification due to mode either being GRAB or UNGRAB");
            return;
        }
        if window == conn.root() {
            // trace!("ignoring focus_out notification due to window being root");
            return;
        }
        if !conn.window_exists(window) {
            return;
        }
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
            .available_rect_after_adding_rects(self.docked.rect_iter());
        match direction {
            Direction::Left => self
                .workspaces
                .get_mut(self.focused_workspace_idx)
                .unwrap()
                .handle_change_focus_window_left(conn, config, &avail_rect),
            Direction::Right => self
                .workspaces
                .get_mut(self.focused_workspace_idx)
                .unwrap()
                .handle_change_focus_window_right(conn, config, &avail_rect),
            Direction::Up => {
                let focused_workspace_id =
                    self.workspaces.get(self.focused_workspace_idx).unwrap().id;
                if focused_workspace_id > 1 {
                    self.handle_switch_to_workspace(conn, config, focused_workspace_id - 1);
                }
            }
            Direction::Down => {
                let focused_workspace_id =
                    self.workspaces.get(self.focused_workspace_idx).unwrap().id;
                if focused_workspace_id < 9 {
                    self.handle_switch_to_workspace(conn, config, focused_workspace_id + 1);
                }
            }
        };
        conn.flush();
    }

    pub fn handle_move_window(&mut self, conn: &Connection, config: &Config, direction: Direction) {
        trace!("move window: {:?}", direction);
        let avail_rect = self
            .rect
            .available_rect_after_adding_rects(self.docked.rect_iter());
        match direction {
            Direction::Left => self
                .workspaces
                .get_mut(self.focused_workspace_idx)
                .unwrap()
                .handle_move_window_left(conn, config, &avail_rect),
            Direction::Right => self
                .workspaces
                .get_mut(self.focused_workspace_idx)
                .unwrap()
                .handle_move_window_right(conn, config, &avail_rect),
            Direction::Up => {
                let focused_workspace_id = self
                    .workspaces
                    .get_mut(self.focused_workspace_idx)
                    .unwrap()
                    .id;
                if focused_workspace_id > 1 {
                    self.handle_move_focused_window_to_workspace(
                        conn,
                        config,
                        focused_workspace_id - 1,
                        true,
                    );
                }
            }
            Direction::Down => {
                let focused_workspace_id = self
                    .workspaces
                    .get_mut(self.focused_workspace_idx)
                    .unwrap()
                    .id;
                if focused_workspace_id < 9 {
                    self.handle_move_focused_window_to_workspace(
                        conn,
                        config,
                        focused_workspace_id + 1,
                        true,
                    );
                }
            }
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
            .available_rect_after_adding_rects(self.docked.rect_iter());
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
            .available_rect_after_adding_rects(self.docked.rect_iter());

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
            self.workspaces.push(Workspace::new(workspace_id, false));
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
        switch_to_new_workspace: bool,
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
            .available_rect_after_adding_rects(self.docked.rect_iter());
        if let Some((window, rect, window_type)) = self
            .workspaces
            .get_mut(self.focused_workspace_idx)
            .unwrap()
            .pop_focused_window(&avail_rect, conn, config)
        {
            let new_focused_workspace_idx = if let Some((idx, _)) = self
                .workspaces
                .iter()
                .enumerate()
                .find(|(_, w)| w.id == workspace_id)
            {
                idx
            } else {
                self.workspaces.push(Workspace::new(workspace_id, false));
                self.workspaces.len() - 1
            };
            match window_type {
                WindowType::Normal => {
                    self.workspaces
                        .get_mut(new_focused_workspace_idx)
                        .unwrap()
                        .handle_existing_normal_window(window, rect, &avail_rect, conn, config);
                }
                WindowType::Floating => {
                    self.workspaces
                        .get_mut(new_focused_workspace_idx)
                        .unwrap()
                        .handle_new_floating_window(window, rect, &avail_rect, conn, config);
                }
                WindowType::Docked => todo!(),
            }
            if switch_to_new_workspace {
                self.workspaces
                    .get_mut(self.focused_workspace_idx)
                    .unwrap()
                    .hide_all_windows(
                        &avail_rect,
                        conn,
                        config,
                        workspace_id < focused_workspace_id,
                    );
                self.focused_workspace_idx = new_focused_workspace_idx;
                self.workspaces
                    .get_mut(self.focused_workspace_idx)
                    .unwrap()
                    .show_all_windows(&avail_rect, conn, config);
            }
            conn.flush();
        }
    }

    pub fn handle_kill_focused_window(&mut self, conn: &Connection, config: &Config) {
        let avail_rect = self
            .rect
            .available_rect_after_adding_rects(self.docked.rect_iter());
        if let Some((window, _, _)) = self
            .workspaces
            .get_mut(self.focused_workspace_idx)
            .unwrap()
            .pop_focused_window(&avail_rect, conn, config)
        {
            trace!("requested to kill focused window {}", window);
            if let Some(_) = self.to_check_deleted.iter().find(|w| w.0 == window) {
                return;
            }
            conn.window_destroy_gracefully(window);
        }
        conn.flush();
    }

    /// destroy windows that still exist after more than 5 seconds since request for graceful deletion was issued
    pub fn check_deleted(&mut self, conn: &Connection) {
        if self.to_check_deleted.is_empty() {
            return;
        }
        let timestamp_now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut index: usize = 0;
        while index < self.to_check_deleted.len() {
            let (window, timestamp_requested) = self.to_check_deleted[index];
            if conn.window_exists(window) {
                if timestamp_now > timestamp_requested && timestamp_now - timestamp_requested >= 5 {
                    conn.window_destroy(window);
                    self.to_check_deleted.remove(index);
                    continue;
                }
            } else {
                self.to_check_deleted.remove(index);
                continue;
            }
            index += 1;
        }
    }

    // #[inline]
    // pub fn cursor_position_within(&self, x: i32, y: i32) -> bool {
    //     self.rect.point_within(x, y)
    // }

    // #[inline]
    // pub fn set_focused_window_under_cursor(
    //     &mut self,
    //     x: i32,
    //     y: i32,
    //     conn: &Connection,
    //     config: &Config,
    // ) {
    //     self.workspaces
    //         .get_mut(self.focused_workspace_idx)
    //         .unwrap()
    //         .set_focused_window_under_cursor(x, y, conn, config);
    // }

    #[inline]
    pub fn handle_enter_notify(
        &mut self,
        window: xcb_window_t,
        conn: &Connection,
        config: &Config,
    ) {
        self.workspaces
            .get_mut(self.focused_workspace_idx)
            .unwrap()
            .handle_enter_notify(window, conn, config);
        conn.flush();
    }

    #[inline]
    pub fn handle_motion_notify(
        &mut self,
        x: i32,
        y: i32,
        window: xcb_window_t,
        state: u32,
        conn: &Connection,
        config: &Config,
    ) {
        let avail_rect = self
            .rect
            .available_rect_after_adding_rects(self.docked.rect_iter());
        self.workspaces
            .get_mut(self.focused_workspace_idx)
            .unwrap()
            .handle_motion_notify(x, y, window, state, conn, config, &avail_rect);
    }

    pub fn handle_button_press(
        &mut self,
        x: i32,
        y: i32,
        window: xcb_window_t,
        state: u16,
        conn: &Connection,
        config: &Config,
    ) {
        self.workspaces
            .get_mut(self.focused_workspace_idx)
            .unwrap()
            .handle_button_press(x, y, window, state, conn, config);
    }

    pub fn handle_button_release(&mut self, conn: &Connection, config: &Config) {
        self.workspaces
            .get_mut(self.focused_workspace_idx)
            .unwrap()
            .handle_button_release(conn, config);
    }
}

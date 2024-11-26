use crate::{
    config::Config,
    monitor::Rect,
    window::{Window, WindowType},
};
use log::{debug, error, info};
use xcb::x;

#[derive(Debug)]
pub struct Workspace {
    pub id: u16,
    tiling_windows: Vec<Window>,
    floating_windows: Vec<Window>,
    available_area: Rect,
    docked_windows: Vec<Window>,
    focused_window: Option<x::Window>,
    focused_window_type: WindowType,
    focused_via_keyboard: bool,
    // config fields
    outer_gap_horiz: u16,
    outer_gap_vert: u16,
    inner_gap: u16,
    border_size: u16,
    hidden: bool,
}

impl Workspace {
    pub fn new(
        id: u16,
        available_area: Rect,
        config: &Config,
        conn: &xcb::Connection,
        hidden: bool,
    ) -> Self {
        let mut tiling_windows = Vec::new();
        tiling_windows.reserve(3);

        let mut res = Self {
            id,
            tiling_windows,
            floating_windows: Vec::new(),
            available_area,
            docked_windows: Vec::new(),
            focused_window: None,
            focused_window_type: WindowType::default(),
            focused_via_keyboard: false,
            outer_gap_horiz: 0,
            outer_gap_vert: 0,
            inner_gap: config.workspaces.inner_gap,
            border_size: config.window.border.size,
            hidden,
        };
        res.update_gaps(config, conn);

        res
    }

    pub fn update_gaps(&mut self, config: &Config, conn: &xcb::Connection) {
        self.border_size = config.window.border.size;

        let mut outer_gap_horiz = config.workspaces.outer_gap_horiz;
        if self.available_area.width <= config.workspaces.outer_gap_horiz {
            error!("Configured horizontal outer gap shouldn't be larger than available area.");
            outer_gap_horiz = 10;
        }

        let mut outer_gap_vert = config.workspaces.outer_gap_vert;
        if self.available_area.height <= config.workspaces.outer_gap_vert {
            error!("Configured vertical outer gap shouldn't be larger than available area.");
            outer_gap_vert = 10;
        }

        let horiz_diff = outer_gap_horiz as i16 - self.outer_gap_horiz as i16;
        let vert_diff = outer_gap_vert as i16 - self.outer_gap_vert as i16;

        self.outer_gap_horiz = outer_gap_horiz;
        self.outer_gap_vert = outer_gap_vert;

        let new_available_area = Rect {
            x: self.available_area.x + horiz_diff,
            y: self.available_area.y + vert_diff,
            width: (self.available_area.width as i16 - horiz_diff * 2) as u16,
            height: (self.available_area.height as i16 - vert_diff * 2) as u16,
        };
        self.update_available_area(new_available_area, conn);
    }

    pub fn update_available_area(&mut self, new_available_area: Rect, conn: &xcb::Connection) {
        if self.available_area == new_available_area {
            return;
        }
        debug!("update_available_area: {:#?}", new_available_area);
        let diff_x = self.available_area.x - new_available_area.x;
        let diff_y = self.available_area.y - new_available_area.y;
        self.available_area = new_available_area;
        self.tiling_windows.iter_mut().for_each(|item| {
            item.rect.x += diff_x;
            item.rect.y += diff_y;
            item.configure(conn);
        });
        self.show_windows_in_available_area(conn);
        self.hide_windows_out_available_area(conn);
    }

    pub fn show_windows_in_available_area(&mut self, conn: &xcb::Connection) {
        self.tiling_windows.iter_mut().for_each(|item| {
            let intersects = item.intersects_with_rect(&self.available_area);
            if intersects && !item.mapped {
                item.map(conn);
            }
        });
    }

    pub fn hide_windows_out_available_area(&mut self, conn: &xcb::Connection) {
        self.tiling_windows.iter_mut().for_each(|item| {
            let intersects = item.intersects_with_rect(&self.available_area);
            if !intersects && item.mapped {
                item.unmap(conn);
            }
        });
    }

    pub fn hide_all_windows(&mut self, monitor_height: u16, conn: &xcb::Connection) {
        if self.hidden {
            return;
        }
        let shift = (monitor_height * self.id) as i16;
        self.tiling_windows.iter_mut().for_each(|w| {
            w.rect.y += shift;
            w.configure(conn);
            if w.mapped {
                w.unmap(conn);
            }
        });
        self.floating_windows.iter_mut().for_each(|w| {
            w.rect.y += shift;
            w.configure(conn);
            if w.mapped {
                w.unmap(conn);
            }
        });
        self.docked_windows.iter_mut().for_each(|w| {
            w.rect.y += shift;
            w.configure(conn);
            if w.mapped {
                w.unmap(conn);
            }
        });
    }

    pub fn unhide_all_windows(&mut self, monitor_height: u16, conn: &xcb::Connection) {
        if !self.hidden {
            return;
        }
        let shift = (monitor_height * self.id) as i16;
        self.tiling_windows.iter_mut().for_each(|w| {
            w.rect.y -= shift;
            w.configure(conn);
            if w.mapped {
                w.map(conn);
            }
        });
        self.floating_windows.iter_mut().for_each(|w| {
            w.rect.y -= shift;
            w.configure(conn);
            if w.mapped {
                w.map(conn);
            }
        });
        self.docked_windows.iter_mut().for_each(|w| {
            w.rect.y -= shift;
            w.configure(conn);
            if w.mapped {
                w.map(conn);
            }
        });
    }

    pub fn set_window_focused(
        &mut self,
        window: &Window,
        conn: &xcb::Connection,
        via_keyboard: bool,
    ) {
        if !via_keyboard && self.focused_via_keyboard {
            return;
        }

        self.focused_window = Some(window.id);
        self.focused_window_type = window.r#type;
        self.focused_via_keyboard = via_keyboard;

        window.set_input_focus(conn);

        debug!(
            "Set focused {:?}, window type: {:?}",
            self.focused_window, self.focused_window_type
        );
    }

    pub fn set_window_focused_by_id(
        &mut self,
        window_id: x::Window,
        conn: &xcb::Connection,
        via_keyboard: bool,
    ) {
        if !via_keyboard && self.focused_via_keyboard {
            return;
        }
        let tiling_window = self.tiling_windows.iter().find(|item| item.id == window_id);
        if let Some(window) = tiling_window {
            window.set_input_focus(conn);
            self.focused_window = Some(window.id);
            self.focused_window_type = window.r#type;
            self.focused_via_keyboard = via_keyboard;
            debug!(
                "Set focused {:?}, window type: {:?}",
                self.focused_window, self.focused_window_type
            );
            return;
        }
        let floating_window = self
            .floating_windows
            .iter()
            .find(|item| item.id == window_id);
        if let Some(window) = floating_window {
            window.set_input_focus(conn);
            self.focused_window = Some(window.id);
            self.focused_window_type = window.r#type;
            self.focused_via_keyboard = via_keyboard;
            debug!(
                "Set focused {:?}, window type: {:?}",
                self.focused_window, self.focused_window_type
            );
            return;
        }
    }

    fn free_rect_after_active_tiling_window(&mut self, expected_width: u16) -> Rect {
        if self.tiling_windows.is_empty() {
            return Rect {
                x: ((self.available_area.x + self.available_area.width as i16) / 2)
                    - (expected_width as i16) / 2,
                y: self.available_area.y,
                width: expected_width,
                height: self.available_area.height,
            };
        }

        if self.focused_window.is_some() && self.focused_window_type == WindowType::Tiling {
            let focused_window_id = self.focused_window.unwrap();
            if let Some((index, focused_window)) = self
                .tiling_windows
                .iter()
                .enumerate()
                .find(|(_, item)| item.id == focused_window_id)
            {
                let mut rect = focused_window.rect.clone();
                rect.x += focused_window.rect.width as i16
                    + self.inner_gap as i16
                    + self.border_size as i16;
                rect.width = expected_width;
                rect.height = self.available_area.height;
                self.tiling_windows[index + 1..]
                    .iter_mut()
                    .for_each(|item| item.rect.x += expected_width as i16);
                return rect;
            }
        }

        let mut rect = self.get_tiling_mapped_last().unwrap().rect.clone();
        rect.x += rect.width as i16 + self.inner_gap as i16 + self.border_size as i16;
        rect.width = expected_width;
        rect.height = self.available_area.height;
        rect
    }

    fn shift_to_make_rect_visible_on_screen(&mut self, rect: &mut Rect) {
        let diff = self.available_area.x + self.available_area.width as i16
            - self.border_size as i16
            - rect.x
            - rect.width as i16;
        debug!("shift_to_make_rect_visible_on_screen - diff: {}", diff);
        if diff < 0 {
            self.tiling_windows.iter_mut().for_each(|item| {
                item.rect.x += diff;
            });
            rect.x += diff;
        }
    }

    #[inline]
    fn get_tiling_mapped_first(&self) -> Option<&Window> {
        self.tiling_windows.iter().find(|item| {
            item.mapped || self.focused_window.is_some() && self.focused_window.unwrap() == item.id
        })
    }

    #[inline]
    fn get_tiling_mapped_last(&self) -> Option<&Window> {
        self.tiling_windows.iter().rev().find(|item| {
            item.mapped || self.focused_window.is_some() && self.focused_window.unwrap() == item.id
        })
    }

    fn shift_to_balance_free_space(&mut self) {
        let left_most_rect = &self.get_tiling_mapped_first().unwrap().rect;
        let free_space_left = left_most_rect.x - self.available_area.x;
        debug!("free_space_left: {}", free_space_left);

        let right_most_rect = &self.get_tiling_mapped_last().unwrap().rect;
        let free_space_right = self.available_area.x + self.available_area.width as i16
            - right_most_rect.x
            - right_most_rect.width as i16;
        debug!("free_space_right: {}", free_space_right);

        if free_space_left < 0 && free_space_right < 0 || free_space_left == free_space_right {
            return;
        }

        let shift = if free_space_right >= 0 && free_space_left >= 0 {
            (free_space_right - free_space_left) / 2
        } else if free_space_left + free_space_right >= 0 {
            -(free_space_left + free_space_right) / 2
        } else if free_space_left > 0 && free_space_right < 0 {
            -free_space_left
        } else {
            0
        };
        debug!("shift: {}", shift);
        if shift != 0 {
            self.tiling_windows
                .iter_mut()
                .for_each(|item| item.rect.x += shift);
        }
    }

    pub fn add_window_tiling(
        &mut self,
        conn: &xcb::Connection,
        window_id: x::Window,
        expected_width: u16,
    ) {
        let mut rect = self.free_rect_after_active_tiling_window(expected_width);
        self.shift_to_make_rect_visible_on_screen(&mut rect);

        self.focused_window = Some(window_id);
        self.focused_window_type = WindowType::Tiling;
        self.focused_via_keyboard = true;

        let window = Window::new(rect, window_id, WindowType::Tiling, self.border_size);
        window.subscribe_to_wm_events(conn);
        self.tiling_windows.push(window);
        self.sort_windows();
        info!("before balance of free space");
        self.tiling_windows
            .iter()
            .for_each(|item| debug!("{:?}", item.rect));
        self.shift_to_balance_free_space();
        info!("after balance of free space");
        self.tiling_windows
            .iter()
            .for_each(|item| debug!("{:?}", item.rect));

        self.tiling_windows.iter().for_each(|item| {
            item.configure(conn);
        });
        self.show_windows_in_available_area(conn);
        self.hide_windows_out_available_area(conn);

        let window = self
            .tiling_windows
            .iter_mut()
            .find(|item| item.id == self.focused_window.unwrap())
            .unwrap();
        window.map(conn);
        window.set_input_focus(conn);
    }

    fn sort_windows(&mut self) {
        self.tiling_windows.sort();
        self.floating_windows.sort();
    }

    pub fn get_window_under_cursor(&self, pos_x: i16, pos_y: i16) -> Option<&Window> {
        let tiling_window = self
            .tiling_windows
            .iter()
            .find(|item| item.point_belongs_to(pos_x, pos_y));
        if tiling_window.is_some() {
            return tiling_window;
        }
        let floating_window = self
            .floating_windows
            .iter()
            .find(|item| item.point_belongs_to(pos_x, pos_y));
        if floating_window.is_some() {
            return floating_window;
        }
        None
    }

    pub fn get_window(&self, window_xcb: x::Window) -> Option<&Window> {
        let tiling_window = self
            .tiling_windows
            .iter()
            .find(|item| item.id == window_xcb);
        if tiling_window.is_some() {
            return tiling_window;
        }
        let floating_window = self
            .floating_windows
            .iter()
            .find(|item| item.id == window_xcb);
        if floating_window.is_some() {
            return floating_window;
        }
        None
    }

    pub fn get_window_mut(&mut self, window_xcb: x::Window) -> Option<&mut Window> {
        let tiling_window = self
            .tiling_windows
            .iter_mut()
            .find(|item| item.id == window_xcb);
        if tiling_window.is_some() {
            return tiling_window;
        }
        let floating_window = self
            .floating_windows
            .iter_mut()
            .find(|item| item.id == window_xcb);
        if floating_window.is_some() {
            return floating_window;
        }
        None
    }

    pub fn get_focused_window(&self) -> Option<&Window> {
        if let Some(focused_window) = self.focused_window {
            if self.focused_window_type == WindowType::Tiling {
                let tiling_window = self
                    .tiling_windows
                    .iter()
                    .find(|item| item.id == focused_window);
                if tiling_window.is_some() {
                    return tiling_window;
                }
            } else {
                let floating_window = self
                    .floating_windows
                    .iter()
                    .find(|item| item.id == focused_window);
                if floating_window.is_some() {
                    return floating_window;
                }
            }
        }
        None
    }

    pub fn get_focused_window_mut(&mut self) -> Option<&mut Window> {
        if let Some(focused_window) = self.focused_window {
            if self.focused_window_type == WindowType::Tiling {
                let tiling_window = self
                    .tiling_windows
                    .iter_mut()
                    .find(|item| item.id == focused_window);
                if tiling_window.is_some() {
                    return tiling_window;
                }
            } else {
                let floating_window = self
                    .floating_windows
                    .iter_mut()
                    .find(|item| item.id == focused_window);
                if floating_window.is_some() {
                    return floating_window;
                }
            }
        }
        None
    }

    #[inline]
    pub fn is_keyboard_focused(&self) -> bool {
        self.focused_via_keyboard
    }

    #[inline]
    pub fn reset_keyboard_focused_flag(&mut self) {
        self.focused_via_keyboard = false;
    }
}

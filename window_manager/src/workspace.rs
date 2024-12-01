use std::ptr;

use crate::{
    config::Config,
    monitor::Rect,
    window::{Window, WindowType},
};
use log::{debug, error};
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
        self.map_windows_in_available_area(conn);
        self.unmap_windows_out_available_area(conn);
    }

    pub fn map_windows_in_available_area(&mut self, conn: &xcb::Connection) {
        self.tiling_windows.iter_mut().for_each(|item| {
            let intersects = item.intersects_with_rect(&self.available_area);
            if intersects && !item.mapped {
                item.map(conn);
            }
        });
    }

    pub fn unmap_windows_out_available_area(&mut self, conn: &xcb::Connection) {
        self.tiling_windows.iter_mut().for_each(|item| {
            let intersects = item.intersects_with_rect(&self.available_area);
            if !intersects && item.mapped {
                item.unmap(conn);
            }
        });
    }

    pub fn hide_all_windows(&mut self, monitor_height: u16, conn: &xcb::Connection) {
        debug!("hide_all_windows at {}, hidden: {}", self.id, self.hidden);
        if self.hidden {
            return;
        }
        if !self.tiling_windows.is_empty()
            || !self.floating_windows.is_empty()
            || !self.docked_windows.is_empty()
        {
            let shift = (monitor_height * self.id) as i16;
            self.tiling_windows.iter_mut().for_each(|w| {
                w.rect.y += shift;
                w.configure(conn);
                if w.mapped {
                    w.hide(conn);
                }
            });
            self.floating_windows.iter_mut().for_each(|w| {
                w.rect.y += shift;
                w.configure(conn);
                if w.mapped {
                    w.hide(conn);
                }
            });
            self.docked_windows.iter_mut().for_each(|w| {
                w.rect.y += shift;
                w.configure(conn);
                if w.mapped {
                    w.hide(conn);
                }
            });
        }
        self.hidden = true;
    }

    pub fn unhide_all_windows(&mut self, monitor_height: u16, conn: &xcb::Connection) {
        debug!("unhide_all_windows at {}, hidden: {}", self.id, self.hidden);
        if !self.hidden {
            return;
        }
        if !self.tiling_windows.is_empty()
            || !self.floating_windows.is_empty()
            || !self.docked_windows.is_empty()
        {
            let shift = (monitor_height * self.id) as i16;
            self.tiling_windows.iter_mut().for_each(|w| {
                w.rect.y -= shift;
                w.configure(conn);
                if w.mapped {
                    w.show(conn);
                }
            });
            self.floating_windows.iter_mut().for_each(|w| {
                w.rect.y -= shift;
                w.configure(conn);
                if w.mapped {
                    w.show(conn);
                }
            });
            self.docked_windows.iter_mut().for_each(|w| {
                w.rect.y -= shift;
                w.configure(conn);
                if w.mapped {
                    w.show(conn);
                }
            });
            self.shift_to_balance_free_space(conn, true);
            self.map_windows_in_available_area(conn);
            self.unmap_windows_out_available_area(conn);
            self.sort_windows();
            self.focused_via_keyboard = true;
        }
        self.hidden = false;
    }

    pub fn set_window_focused(
        &mut self,
        window: &Window,
        conn: &xcb::Connection,
        via_keyboard: bool,
        config: &Config,
    ) {
        if !via_keyboard && self.focused_via_keyboard {
            return;
        }

        window.set_input_focus(conn);
        self.unset_focused_border(conn, config);
        window.change_border_color(conn, config.window.border.color_active_u32.unwrap());

        self.focused_window = Some(window.id);
        self.focused_window_type = window.r#type;
        self.focused_via_keyboard = via_keyboard;

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
        config: &Config,
    ) {
        if !via_keyboard && self.focused_via_keyboard {
            return;
        }
        if let Some(window) = self.tiling_windows.iter().find(|item| item.id == window_id) {
            window.set_input_focus(conn);
            self.unset_focused_border(conn, config);
            window.change_border_color(conn, config.window.border.color_active_u32.unwrap());

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
            self.unset_focused_border(conn, config);
            window.change_border_color(conn, config.window.border.color_active_u32.unwrap());

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

    fn unset_focused_border(&self, conn: &xcb::Connection, config: &Config) {
        if let Some(focused_id) = self.focused_window {
            if self.focused_window_type == WindowType::Tiling {
                if let Some(focused_window) =
                    self.tiling_windows.iter().find(|w| w.id == focused_id)
                {
                    focused_window.change_border_color(
                        conn,
                        config.window.border.color_inactive_u32.unwrap(),
                    );
                }
            } else if self.focused_window_type == WindowType::Floating {
                if let Some(focused_window) =
                    self.floating_windows.iter().find(|w| w.id == focused_id)
                {
                    focused_window.change_border_color(
                        conn,
                        config.window.border.color_inactive_u32.unwrap(),
                    );
                }
            }
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
                    .for_each(|item| {
                        item.rect.x += (expected_width + self.border_size + self.inner_gap) as i16
                    });
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

    fn shift_to_balance_free_space(&mut self, conn: &xcb::Connection, configure: bool) {
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
                .for_each(|w| w.rect.x += shift);
            if configure {
                self.tiling_windows
                    .iter_mut()
                    .for_each(|w| w.configure(conn));
            }
        }
    }

    pub fn add_tiling_window_by_id_width(
        &mut self,
        conn: &xcb::Connection,
        window_id: x::Window,
        expected_width: u16,
        config: &Config,
        monitor_height: u16,
    ) {
        let mut rect = self.free_rect_after_active_tiling_window(expected_width);
        self.shift_to_make_rect_visible_on_screen(&mut rect);
        if self.hidden {
            rect.y += (monitor_height * self.id) as i16;
        }

        let mut window = Window::new(rect, window_id, WindowType::Tiling, self.border_size);
        window.configure(conn);
        window.subscribe_to_wm_events(conn);
        if !self.hidden {
            window.map(conn);
            self.set_window_focused(&window, conn, true, config);
        } else {
            self.focused_window = Some(window.id);
            self.focused_window_type = window.r#type;
            self.focused_via_keyboard = true;
        }

        self.tiling_windows.push(window);
        self.sort_windows();

        self.shift_to_balance_free_space(conn, false);
        self.tiling_windows.iter().for_each(|item| {
            item.configure(conn);
        });
        if !self.hidden {
            self.map_windows_in_available_area(conn);
            self.unmap_windows_out_available_area(conn);
        }
    }

    pub fn add_existing_tiling_window(
        &mut self,
        conn: &xcb::Connection,
        window: Window,
        config: &Config,
        monitor_height: u16,
    ) {
        let mut rect = self.free_rect_after_active_tiling_window(window.rect.width);
        self.shift_to_make_rect_visible_on_screen(&mut rect);
        if self.hidden {
            rect.y = self.available_area.y + (monitor_height * self.id) as i16;
        }

        let mut window = window;
        window.rect = rect;

        window.map(conn);
        self.set_window_focused(&window, conn, true, config);
        if self.hidden {
            window.hide(conn);
        }
        // if !self.hidden {
        //     window.map(conn);
        //     self.set_window_focused(&window, conn, true, config);
        // } else {
        //     window.hide(conn);
        //     self.focused_window = Some(window.id);
        //     self.focused_window_type = window.r#type;
        //     self.focused_via_keyboard = true;
        // }

        self.tiling_windows.push(window);
        self.sort_windows();

        self.shift_to_balance_free_space(conn, false);
        self.tiling_windows.iter().for_each(|item| {
            item.configure(conn);
        });
        if !self.hidden {
            self.map_windows_in_available_area(conn);
            self.unmap_windows_out_available_area(conn);
        }
    }

    pub fn shift_focus_left(&mut self, conn: &xcb::Connection, config: &Config) {
        if let Some(focused_id) = self.focused_window {
            if self.focused_window_type == WindowType::Tiling {
                if let Some((index, _)) = self
                    .tiling_windows
                    .iter()
                    .enumerate()
                    .find(|(_, w)| w.id == focused_id)
                {
                    if index != 0 {
                        let new_selected_window_rect = self.tiling_windows[index - 1].rect.clone();
                        let new_selected_window_id = self.tiling_windows[index - 1].id;
                        self.set_window_focused_by_id(new_selected_window_id, conn, true, config);

                        let diff = new_selected_window_rect.x
                            - self.available_area.x
                            - self.border_size as i16;
                        if diff < 0 {
                            self.tiling_windows.iter_mut().for_each(|w| {
                                w.rect.x -= diff;
                                w.configure(conn);
                            });
                            self.map_windows_in_available_area(conn);
                            self.unmap_windows_out_available_area(conn);
                        }
                    }
                }
            }
        }
    }

    pub fn shift_focus_right(&mut self, conn: &xcb::Connection, config: &Config) {
        if let Some(focused_id) = self.focused_window {
            if self.focused_window_type == WindowType::Tiling {
                if let Some((index, _)) = self
                    .tiling_windows
                    .iter()
                    .enumerate()
                    .find(|(_, w)| w.id == focused_id)
                {
                    if index != self.tiling_windows.len() - 1 {
                        let new_selected_window_rect = self.tiling_windows[index + 1].rect.clone();
                        let new_selected_window_id = self.tiling_windows[index + 1].id;
                        self.set_window_focused_by_id(new_selected_window_id, conn, true, config);

                        let diff = self.available_area.x + self.available_area.width as i16
                            - self.border_size as i16
                            - new_selected_window_rect.x
                            - new_selected_window_rect.width as i16;
                        if diff < 0 {
                            self.tiling_windows.iter_mut().for_each(|w| {
                                w.rect.x += diff;
                                w.configure(conn);
                            });
                            self.map_windows_in_available_area(conn);
                            self.unmap_windows_out_available_area(conn);
                        }
                    }
                }
            }
        }
    }

    fn swap_windows_rects(
        &mut self,
        index_current: usize,
        index_swapped_with: usize,
        conn: &xcb::Connection,
        config: &Config,
    ) {
        assert!(index_current != index_swapped_with);
        unsafe {
            let lhs: *mut Rect = &mut self.tiling_windows.get_mut(index_current).unwrap().rect;
            let rhs: *mut Rect = &mut self
                .tiling_windows
                .get_mut(index_swapped_with)
                .unwrap()
                .rect;
            ptr::swap(lhs, rhs);
        }
        self.sort_windows();

        let old_index_current = index_current;
        let index_current = index_swapped_with;
        let index_swapped_with = old_index_current;

        let current = self.tiling_windows.get(index_current).unwrap();
        let swapped_with = self.tiling_windows.get(index_swapped_with).unwrap();

        current.configure(conn);
        swapped_with.configure(conn);

        swapped_with.change_border_color(conn, config.window.border.color_inactive_u32.unwrap());
        current.change_border_color(conn, config.window.border.color_active_u32.unwrap());
        current.set_input_focus(conn);
        self.focused_via_keyboard = true;

        let sign = if index_current > index_swapped_with {
            -1
        } else {
            1
        };
        let diff = if index_current > index_swapped_with {
            swapped_with.rect.x - self.available_area.x - self.border_size as i16
        } else {
            self.available_area.x + self.available_area.width as i16
                - self.border_size as i16
                - swapped_with.rect.x
                - swapped_with.rect.width as i16
        };
        if diff < 0 {
            self.tiling_windows.iter_mut().for_each(|w| {
                w.rect.x += diff * sign;
                w.configure(conn);
            });
            self.map_windows_in_available_area(conn);
            self.unmap_windows_out_available_area(conn);
        }
    }

    pub fn move_window_left(&mut self, conn: &xcb::Connection, config: &Config) {
        if let Some(focused_id) = self.focused_window {
            if self.focused_window_type == WindowType::Tiling {
                let index = self
                    .tiling_windows
                    .iter()
                    .enumerate()
                    .find(|(_, w)| w.id == focused_id)
                    .unwrap()
                    .0;
                if index != 0 {
                    self.swap_windows_rects(index, index - 1, conn, config);
                }
            }
        }
    }

    pub fn move_window_right(&mut self, conn: &xcb::Connection, config: &Config) {
        if let Some(focused_id) = self.focused_window {
            if self.focused_window_type == WindowType::Tiling {
                let index = self
                    .tiling_windows
                    .iter()
                    .enumerate()
                    .find(|(_, w)| w.id == focused_id)
                    .unwrap()
                    .0;
                if index != self.tiling_windows.len() - 1 {
                    self.swap_windows_rects(index, index + 1, conn, config);
                }
            }
        }
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

    pub fn get_focused_window_type(&self) -> Option<WindowType> {
        if self.focused_window.is_some() {
            Some(self.focused_window_type)
        } else {
            None
        }
    }

    pub fn grow_width_selected_window(&mut self, conn: &xcb::Connection, pixels: u16) {
        if let Some(focused_window_id) = self.focused_window {
            if self.focused_window_type == WindowType::Tiling {
                let focused_window_index = self
                    .tiling_windows
                    .iter()
                    .enumerate()
                    .find(|(_, w)| w.id == focused_window_id)
                    .unwrap()
                    .0;
                self.tiling_windows[focused_window_index].rect.width += pixels;
                self.tiling_windows[..=focused_window_index]
                    .iter_mut()
                    .for_each(|w| {
                        w.rect.x -= (pixels / 2) as i16;
                        w.configure(conn);
                    });
                self.tiling_windows[focused_window_index + 1..]
                    .iter_mut()
                    .for_each(|w| {
                        w.rect.x += (pixels / 2) as i16;
                        w.configure(conn);
                    });
            }
        }
    }

    pub fn shrink_width_selected_window(
        &mut self,
        conn: &xcb::Connection,
        pixels: u16,
        config: &Config,
    ) {
        if let Some(focused_window_id) = self.focused_window {
            if self.focused_window_type == WindowType::Tiling {
                let focused_window_index = self
                    .tiling_windows
                    .iter()
                    .enumerate()
                    .find(|(_, w)| w.id == focused_window_id)
                    .unwrap()
                    .0;
                if self.tiling_windows[focused_window_index].rect.width as i16 - pixels as i16
                    <= config.window.minimum_width_tiling as i16
                {
                    return;
                }
                self.tiling_windows[focused_window_index].rect.width -= pixels;
                self.tiling_windows[..=focused_window_index]
                    .iter_mut()
                    .for_each(|w| {
                        w.rect.x += (pixels / 2) as i16;
                        w.configure(conn);
                    });
                self.tiling_windows[focused_window_index + 1..]
                    .iter_mut()
                    .for_each(|w| {
                        w.rect.x -= (pixels / 2) as i16;
                        w.configure(conn);
                    });
            }
        }
    }

    pub fn remove_selected_window(
        &mut self,
        conn: &xcb::Connection,
        config: &Config,
        monitor_height: u16,
    ) -> Option<Window> {
        if let Some(focused_window_id) = self.focused_window {
            if self.focused_window_type == WindowType::Tiling {
                let focused_window_index = self
                    .tiling_windows
                    .iter()
                    .enumerate()
                    .find(|(_, w)| w.id == focused_window_id)
                    .unwrap()
                    .0;
                let removed_window = self.tiling_windows.remove(focused_window_index);
                if focused_window_index < self.tiling_windows.len() {
                    self.tiling_windows[focused_window_index..]
                        .iter_mut()
                        .for_each(|w| {
                            w.rect.x -= (removed_window.rect.width
                                + self.inner_gap
                                + self.border_size) as i16;
                            w.configure(conn);
                        });
                    self.map_windows_in_available_area(conn);
                    self.unmap_windows_out_available_area(conn);
                    self.set_window_focused_by_id(
                        self.tiling_windows[focused_window_index].id,
                        conn,
                        true,
                        config,
                    );
                    let mut focused_rect = self.tiling_windows[focused_window_index].rect.clone();
                    self.shift_to_make_rect_visible_on_screen(&mut focused_rect);
                    self.tiling_windows[focused_window_index].rect = focused_rect;
                } else if !self.tiling_windows.is_empty() {
                    self.set_window_focused_by_id(
                        self.tiling_windows.last().unwrap().id,
                        conn,
                        true,
                        config,
                    );
                } else {
                    self.focused_window = None;
                }
                if !self.tiling_windows.is_empty() {
                    self.shift_to_balance_free_space(conn, true);
                    self.map_windows_in_available_area(conn);
                    self.unmap_windows_out_available_area(conn);
                    if config.switch_workspace_on_window_workspace_change {
                        self.hide_all_windows(monitor_height, conn);
                    }
                }
                return Some(removed_window);
            }
        }
        None
    }
}

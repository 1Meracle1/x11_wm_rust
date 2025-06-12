use std::i32;

use base::Rect;
use log::{trace, warn};
use x11_bindings::{
    bindings::{
        XCB_BUTTON_MASK_1, XCB_BUTTON_MASK_3, XCB_CW_BORDER_PIXEL, XCB_CW_EVENT_MASK,
        XCB_EVENT_MASK_BUTTON_MOTION, XCB_EVENT_MASK_BUTTON_RELEASE, XCB_EVENT_MASK_ENTER_WINDOW,
        XCB_EVENT_MASK_FOCUS_CHANGE, XCB_EVENT_MASK_LEAVE_WINDOW, XCB_EVENT_MASK_POINTER_MOTION,
        XCB_ICCCM_SIZE_HINT_BASE_SIZE, XCB_ICCCM_SIZE_HINT_P_MAX_SIZE,
        XCB_ICCCM_SIZE_HINT_P_MIN_SIZE, XCB_ICCCM_SIZE_HINT_P_POSITION, XCB_ICCCM_SIZE_HINT_P_SIZE,
        XCB_MOD_MASK_1, xcb_button_t, xcb_size_hints_t, xcb_timestamp_t, xcb_window_t,
    },
    connection::WindowType,
};

use crate::{config::Config, connection::Connection, window::WindowsCollection};

#[derive(Debug)]
pub struct Workspace {
    pub id: u32,
    pub normal: WindowsCollection,
    floating: WindowsCollection,
    docked: WindowsCollection,
    focused_idx: usize,
    focused_type: WindowType,
    is_visible: bool,
    focused_via_keyboard: bool,
    window_mouse_interaction: Option<WindowMouseInteraction>,
}

impl Workspace {
    pub fn new(id: u32, is_visible: bool) -> Self {
        Self {
            id,
            normal: WindowsCollection::new(5),
            floating: WindowsCollection::new(3),
            docked: WindowsCollection::new(0),
            focused_idx: 0,
            focused_type: WindowType::Normal,
            is_visible,
            focused_via_keyboard: false,
            window_mouse_interaction: None,
        }
    }

    pub fn handle_existing_floating_window(
        &mut self,
        window: xcb_window_t,
        rect: &Rect,
        monitor_rect: &Rect,
        conn: &Connection,
        config: &Config,
    ) {
        // conn.window_configure(window, &window_rect, config.border_size);
        // // self.apply_rounded_mask(window, window_rect.width, window_rect.height, conn);

        // conn.change_window_attrs(
        //     window,
        //     XCB_CW_EVENT_MASK,
        //     XCB_EVENT_MASK_FOCUS_CHANGE
        //         | XCB_EVENT_MASK_ENTER_WINDOW
        //         | XCB_EVENT_MASK_LEAVE_WINDOW
        //         | XCB_EVENT_MASK_BUTTON_MOTION,
        // );

        // conn.map_window(window);

        // self.floating.add(window, window_rect, true);
        // self.set_focused(window, WindowType::Floating, conn, config);
        // self.focused_via_keyboard = true;
        // self.reset_window_interaction_state(conn);
    }

    pub fn handle_new_floating_window(
        &mut self,
        window: xcb_window_t,
        rect_hints: Option<xcb_size_hints_t>,
        monitor_rect: &Rect,
        conn: &Connection,
        config: &Config,
    ) {
        let center_x = (monitor_rect.x + monitor_rect.width as i32) / 2;
        let center_y = (monitor_rect.y + monitor_rect.height as i32) / 2;
        let mut rect = {
            let width: u32 = 800;
            let height: u32 = 600;
            Rect {
                x: center_x - (width / 2) as i32,
                y: center_y - (height / 2) as i32,
                width,
                height,
            }
        };

        let mut rect_hints_relevant = false;
        if let Some(hints) = rect_hints {
            if hints.flags & XCB_ICCCM_SIZE_HINT_P_POSITION != 0 {
                rect.x = hints.x;
                rect.y = hints.y;
                rect_hints_relevant = true;
            }
            if hints.flags & XCB_ICCCM_SIZE_HINT_P_SIZE != 0 && hints.width > 0 && hints.height > 0
            {
                rect.width = hints.width as u32;
                rect.height = hints.height as u32;
                rect_hints_relevant = true;
            } else if hints.flags & XCB_ICCCM_SIZE_HINT_P_MIN_SIZE != 0
                && hints.min_width > 0
                && hints.min_height > 0
            {
                rect.width = hints.min_width as u32;
                rect.height = hints.min_height as u32;
                rect_hints_relevant = true;
            } else if hints.flags & XCB_ICCCM_SIZE_HINT_P_MAX_SIZE != 0
                && hints.max_width > 0
                && hints.max_height > 0
            {
                rect.width = hints.max_width as u32;
                rect.height = hints.max_height as u32;
                rect_hints_relevant = true;
            } else if hints.flags & XCB_ICCCM_SIZE_HINT_BASE_SIZE != 0
                && hints.base_width > 0
                && hints.base_height > 0
            {
                rect.width = hints.base_width as u32;
                rect.height = hints.base_height as u32;
                rect_hints_relevant = true;
            }
        }
        if !rect_hints_relevant {
            // check if there are any floating windows that already have the same upper-left corner's position
            // move them slightly lower and to the right, though not in the case it is lower and/or right-er than accepted
            self.floating.sort_by_rect_x_asc();
            self.floating.sort_by_rect_y_asc();
            let mut adjusted_rect = rect.clone();
            for other_rect in self.floating.rect_iter() {
                if other_rect.x == rect.x && other_rect.y == rect.y {
                    adjusted_rect.x += 20;
                    adjusted_rect.y += 20;

                    if adjusted_rect.x
                        + (adjusted_rect.width + config.border_size + config.outer_gap_horiz) as i32
                        >= (monitor_rect.x + monitor_rect.width as i32)
                        || adjusted_rect.y
                            + (adjusted_rect.height + config.border_size + config.outer_gap_vert)
                                as i32
                            >= (monitor_rect.y + monitor_rect.height as i32)
                    {
                        adjusted_rect = rect.clone();
                        break;
                    }
                }
            }
            rect = adjusted_rect;
        }

        conn.window_configure(window, &rect, config.border_size);

        conn.change_window_attrs(
            window,
            XCB_CW_EVENT_MASK,
            XCB_EVENT_MASK_FOCUS_CHANGE
                | XCB_EVENT_MASK_ENTER_WINDOW
                | XCB_EVENT_MASK_LEAVE_WINDOW
                | XCB_EVENT_MASK_BUTTON_MOTION,
        );

        conn.map_window(window);

        conn.grab_button(window);

        self.floating.add(window, rect, true);
        self.set_focused(window, WindowType::Floating, conn, config);
        self.focused_via_keyboard = true;
        self.reset_window_interaction_state(conn);
    }

    // pub fn raise_all_floating_windows(&self, conn: &Connection) {
    //     for (&window, _, &visible) in self.floating.iter() {
    //         if visible {
    //             conn.window_raise(window);
    //         }
    //     }
    // }

    pub fn handle_new_normal_window(
        &mut self,
        window: xcb_window_t,
        monitor_rect: &Rect,
        conn: &Connection,
        config: &Config,
    ) {
        let avail_rect = self.available_rectangle(monitor_rect, config);
        let window_width = ((avail_rect.x + avail_rect.width as i32) as f64
            * config.default_screen_width_percent_tiling) as u32
            - config.border_size * 2;
        let window_height = avail_rect.height - config.border_size * 2;
        let focused_idx =
            if self.focused_type == WindowType::Normal && self.focused_idx < self.normal.len() {
                Some(self.focused_idx)
            } else {
                None
            };

        let (window_rect, move_left_rects_by, move_right_rects_by) = avail_rect
            .calc_new_rect_added_after_focused(
                window_width,
                window_height,
                focused_idx,
                self.normal.rects(),
                config.inner_gap + config.border_size * 2,
            );
        if move_left_rects_by != 0 {
            if let Some(focused_idx) = focused_idx {
                for rect in self.normal.rects_slice_mut(..=focused_idx).iter_mut() {
                    rect.x += move_left_rects_by;
                }
            } else {
                for rect in self.normal.rect_iter_mut() {
                    rect.x += move_left_rects_by;
                }
            }
        }
        if move_right_rects_by != 0 {
            if let Some(focused_idx) = focused_idx {
                for rect in self.normal.rects_slice_mut(focused_idx + 1..).iter_mut() {
                    rect.x += move_right_rects_by;
                }
            }
        }

        conn.change_window_attrs(
            window,
            XCB_CW_EVENT_MASK,
            XCB_EVENT_MASK_FOCUS_CHANGE | XCB_EVENT_MASK_ENTER_WINDOW | XCB_EVENT_MASK_LEAVE_WINDOW,
        );

        self.normal.add(window, window_rect, false);
        self.normal.sort_by_rect_x_asc();
        self.normal
            .iter()
            .for_each(|(w, rect, _)| conn.window_configure(*w, rect, config.border_size));
        self.fix_windows_visibility(&avail_rect, conn);
        self.set_focused(window, WindowType::Normal, conn, config);
        self.focused_via_keyboard = true;
        self.reset_window_interaction_state(conn);
    }

    pub fn handle_existing_normal_window(
        &mut self,
        window: xcb_window_t,
        window_rect: Rect,
        monitor_rect: &Rect,
        conn: &Connection,
        config: &Config,
    ) {
        let avail_rect = self.available_rectangle(monitor_rect, config);
        let focused_idx =
            if self.focused_type == WindowType::Normal && self.focused_idx < self.normal.len() {
                Some(self.focused_idx)
            } else {
                None
            };

        let (window_rect, move_left_rects_by, move_right_rects_by) = avail_rect
            .calc_new_rect_added_after_focused(
                window_rect.width,
                window_rect.height,
                focused_idx,
                self.normal.rects(),
                config.inner_gap + config.border_size * 2,
            );
        if move_left_rects_by != 0 {
            if let Some(focused_idx) = focused_idx {
                for rect in self.normal.rects_slice_mut(..=focused_idx).iter_mut() {
                    rect.x += move_left_rects_by;
                }
            } else {
                for rect in self.normal.rect_iter_mut() {
                    rect.x += move_left_rects_by;
                }
            }
        }
        if move_right_rects_by != 0 {
            if let Some(focused_idx) = focused_idx {
                for rect in self.normal.rects_slice_mut(focused_idx + 1..).iter_mut() {
                    rect.x += move_right_rects_by;
                }
            }
        }

        self.normal.add(window, window_rect, !self.is_visible);
        self.normal.sort_by_rect_x_asc();
        self.normal
            .iter()
            .for_each(|(w, rect, _)| conn.window_configure(*w, rect, config.border_size));
        if self.is_visible {
            self.fix_windows_visibility(monitor_rect, conn);
        } else {
            conn.unmap_window(window);
        }
        self.set_focused(window, WindowType::Normal, conn, config);
        self.focused_via_keyboard = true;
        self.reset_window_interaction_state(conn);
    }

    pub fn handle_new_docked_window(
        &mut self,
        window: xcb_window_t,
        partial_strut: &Rect,
        monitor_rect: &Rect,
        conn: &Connection,
    ) {
        let magnified_rect = monitor_rect.new_rect_magnified(&partial_strut);
        conn.window_configure(window, &magnified_rect, 0);
        conn.map_window(window);
        self.docked.add(window, magnified_rect, true);
        self.reset_window_interaction_state(conn);
    }

    pub fn handle_change_focus_window_left(
        &mut self,
        conn: &Connection,
        config: &Config,
        monitor_rect: &Rect,
    ) {
        match self.focused_type {
            WindowType::Normal if self.focused_idx < self.normal.len() => {
                if self.focused_idx > 0 {
                    let new_focused_idx = self.focused_idx - 1;
                    let avail_rect = self.available_rectangle(monitor_rect, config);
                    let move_right_x = {
                        let focused_rect = self.normal.at_rect(new_focused_idx).unwrap();
                        if focused_rect.x < avail_rect.x {
                            avail_rect.x - focused_rect.x
                        } else {
                            0
                        }
                    };
                    if move_right_x > 0 {
                        for (window, rect, _) in self.normal.iter_mut() {
                            rect.x += move_right_x;
                            conn.window_configure(*window, rect, config.border_size);
                        }
                        self.fix_windows_visibility(monitor_rect, conn);
                    }
                    self.set_focused_by_index(new_focused_idx, self.focused_type, conn, config);
                    self.focused_via_keyboard = true;
                    self.reset_window_interaction_state(conn);
                }
            }
            WindowType::Floating => {
                trace!("change focus window left event received when floating window is focused");
            }
            WindowType::Docked => {}
            _ => {
                warn!("change focus window left when no window is focused");
            }
        }
    }

    pub fn handle_change_focus_window_right(
        &mut self,
        conn: &Connection,
        config: &Config,
        monitor_rect: &Rect,
    ) {
        match self.focused_type {
            WindowType::Normal if self.focused_idx < self.normal.len() => {
                if self.focused_idx < self.normal.len() - 1 {
                    let new_focused_idx = self.focused_idx + 1;
                    let avail_rect = self.available_rectangle(monitor_rect, config);
                    let move_left_x = {
                        let focused_rect = self.normal.at_rect(new_focused_idx).unwrap();
                        if focused_rect.x + focused_rect.width as i32
                            > avail_rect.x + avail_rect.width as i32
                        {
                            focused_rect.x + focused_rect.width as i32
                                - avail_rect.x
                                - avail_rect.width as i32
                        } else {
                            0
                        }
                    };
                    if move_left_x > 0 {
                        for (window, rect, _) in self.normal.iter_mut() {
                            rect.x -= move_left_x;
                            conn.window_configure(*window, rect, config.border_size);
                        }
                        self.fix_windows_visibility(monitor_rect, conn);
                    }
                    self.set_focused_by_index(new_focused_idx, self.focused_type, conn, config);
                    self.focused_via_keyboard = true;
                    self.reset_window_interaction_state(conn);
                }
            }
            WindowType::Floating => {
                trace!("change focus window left event received when floating window is focused");
            }
            WindowType::Docked => {}
            _ => {
                warn!("change focus window left when no window is focused");
            }
        }
    }

    pub fn handle_move_window_left(
        &mut self,
        conn: &Connection,
        config: &Config,
        monitor_rect: &Rect,
    ) {
        match self.focused_type {
            WindowType::Normal if self.focused_idx < self.normal.len() => {
                if self.focused_idx > 0 {
                    let currently_focused_idx = self.focused_idx;
                    let target_idx = self.focused_idx - 1;
                    let avail_rect = self.available_rectangle(monitor_rect, config);

                    self.normal.swap_windows(target_idx, currently_focused_idx);
                    self.normal.swap_visibles(target_idx, currently_focused_idx);

                    {
                        let current_width = self.normal.index_rect(currently_focused_idx).width;
                        let target_width = self.normal.index_rect(target_idx).width;
                        self.normal.index_rect_mut(target_idx).width = current_width;
                        self.normal.index_rect_mut(currently_focused_idx).width = target_width;
                        if current_width != target_width {
                            self.normal.index_rect_mut(currently_focused_idx).x +=
                                current_width as i32 - target_width as i32;
                        }
                    }
                    self.focused_idx = target_idx;

                    let (win_newly_focused, rect_newly_focused) = self.normal.index(target_idx);

                    let (win_prev_focused, rect_prev_focused) =
                        self.normal.index(currently_focused_idx);

                    // adjust viewport if required
                    let move_right_x = {
                        let focused_rect_after_swap = self.normal.index_rect(self.focused_idx);
                        if focused_rect_after_swap.x < avail_rect.x {
                            avail_rect.x - focused_rect_after_swap.x
                        } else {
                            0
                        }
                    };
                    if move_right_x != 0 {
                        for (window, rect, _) in self.normal.iter_mut() {
                            rect.x += move_right_x;
                            conn.window_configure(*window, &rect, config.border_size);
                        }
                        self.fix_windows_visibility(monitor_rect, conn);
                    } else {
                        conn.window_configure(
                            win_newly_focused,
                            rect_newly_focused,
                            config.border_size,
                        );
                        conn.window_configure(
                            win_prev_focused,
                            rect_prev_focused,
                            config.border_size,
                        );
                    }
                    self.focused_via_keyboard = true;
                    self.reset_window_interaction_state(conn);
                }
            }
            WindowType::Floating => {
                trace!("move window left event received when floating window is focused");
            }
            WindowType::Docked => {}
            _ => {
                warn!("move window left when docked window is focused");
            }
        }
    }

    pub fn handle_move_window_right(
        &mut self,
        conn: &Connection,
        config: &Config,
        monitor_rect: &Rect,
    ) {
        match self.focused_type {
            WindowType::Normal if self.focused_idx < self.normal.len() => {
                if self.focused_idx < self.normal.len() - 1 {
                    let currently_focused_idx = self.focused_idx;
                    let target_idx = self.focused_idx + 1;
                    let avail_rect = self.available_rectangle(monitor_rect, config);

                    self.normal.swap_windows(target_idx, currently_focused_idx);
                    self.normal.swap_visibles(target_idx, currently_focused_idx);

                    {
                        let current_width = self.normal.index_rect(currently_focused_idx).width;
                        let target_width = self.normal.index_rect(target_idx).width;
                        self.normal.index_rect_mut(target_idx).width = current_width;
                        self.normal.index_rect_mut(currently_focused_idx).width = target_width;
                        if current_width != target_width {
                            self.normal.index_rect_mut(target_idx).x +=
                                target_width as i32 - current_width as i32;
                        }
                    }
                    self.focused_idx = target_idx;

                    let (win_newly_focused, rect_newly_focused) = self.normal.index(target_idx);

                    let (win_prev_focused, rect_prev_focused) =
                        self.normal.index(currently_focused_idx);

                    // adjust viewport if required
                    let move_left_x = {
                        if rect_newly_focused.x + rect_newly_focused.width as i32
                            > avail_rect.x + avail_rect.width as i32
                        {
                            rect_newly_focused.x + rect_newly_focused.width as i32
                                - avail_rect.x
                                - avail_rect.width as i32
                        } else {
                            0
                        }
                    };
                    if move_left_x > 0 {
                        for (window, rect, _) in self.normal.iter_mut() {
                            rect.x -= move_left_x;
                            conn.window_configure(*window, rect, config.border_size);
                        }
                        self.fix_windows_visibility(monitor_rect, conn);
                    } else {
                        conn.window_configure(
                            win_newly_focused,
                            rect_newly_focused,
                            config.border_size,
                        );
                        conn.window_configure(
                            win_prev_focused,
                            rect_prev_focused,
                            config.border_size,
                        );
                    }
                    self.focused_via_keyboard = true;
                    self.reset_window_interaction_state(conn);
                }
            }
            WindowType::Floating => {
                trace!("move window left event received when floating window is focused");
            }
            WindowType::Docked => {}
            _ => {
                warn!("move window left when docked window is focused");
            }
        }
    }

    fn set_focused(
        &mut self,
        window: xcb_window_t,
        window_type: WindowType,
        conn: &Connection,
        config: &Config,
    ) {
        let index = {
            match window_type {
                WindowType::Normal => self.normal.index_of(window).unwrap(),
                WindowType::Floating => self.floating.index_of(window).unwrap(),
                WindowType::Docked => self.docked.index_of(window).unwrap(),
            }
        };
        self.set_focused_by_index_window(index, window, window_type, conn, config);
    }

    fn set_focused_by_index(
        &mut self,
        index: usize,
        window_type: WindowType,
        conn: &Connection,
        config: &Config,
    ) {
        if let Some(window) = {
            match window_type {
                WindowType::Normal => self.normal.at_window(index),
                WindowType::Floating => self.floating.at_window(index),
                WindowType::Docked => self.docked.at_window(index),
            }
        } {
            self.set_focused_by_index_window(index, window, window_type, conn, config);
        }
    }

    fn set_focused_by_index_window(
        &mut self,
        index: usize,
        window: xcb_window_t,
        window_type: WindowType,
        conn: &Connection,
        config: &Config,
    ) {
        self.focused_idx = index;
        self.focused_type = window_type;
        conn.change_window_attrs(
            window,
            XCB_CW_BORDER_PIXEL,
            config.border_color_active_int.unwrap(),
        );
        if self.is_visible {
            trace!("apply focus to {}", window);
            conn.window_set_input_focus(window);
            if window_type == WindowType::Floating {
                conn.window_raise(window);
            }
        }
    }

    pub fn handle_resize_window_horizontal(
        &mut self,
        conn: &Connection,
        config: &Config,
        monitor_rect: &Rect,
        size_change_pixels: i32,
    ) {
        let avail_rect = self.available_rectangle(monitor_rect, config);
        match self.focused_type {
            WindowType::Normal => {
                if self.normal.is_empty() {
                    return
                }
                if self.focused_idx >= self.normal.len() {
                    self.focused_idx = self.normal.len() - 1;
                }
                let focused_rect = self.normal.index_rect(self.focused_idx).clone();
                let new_width = (focused_rect.width as i32 + size_change_pixels)
                    .clamp(config.minimum_width_tiling as i32, avail_rect.width as i32);
                let new_x = (focused_rect.x - size_change_pixels / 2)
                    .clamp(avail_rect.x, avail_rect.x + avail_rect.width as i32);

                self.normal.update_rect_at(
                    self.focused_idx,
                    Rect {
                        x: new_x,
                        y: focused_rect.y,
                        width: new_width as u32,
                        height: focused_rect.height,
                    },
                );

                let move_x_from_left = new_x - focused_rect.x;
                let move_x_from_right = move_x_from_left + new_width - focused_rect.width as i32;
                if move_x_from_left != 0 {
                    for rect in self.normal.rects_slice_mut(..self.focused_idx).iter_mut() {
                        rect.x += move_x_from_left;
                    }
                }
                if move_x_from_right != 0 {
                    for rect in self
                        .normal
                        .rects_slice_mut(self.focused_idx + 1..)
                        .iter_mut()
                    {
                        rect.x += move_x_from_right;
                    }
                }
                for (window, rect, _) in self.normal.iter() {
                    conn.window_configure(*window, rect, config.border_size);
                }

                self.fix_windows_visibility(monitor_rect, conn);
                self.focused_via_keyboard = true;
                self.reset_window_interaction_state(conn);
            }
            WindowType::Floating => {
                trace!("horizontal resize window event received when floating window is focused");
            }
            WindowType::Docked => warn!("horizontal resize window when docked window is focused"),
        }
    }

    pub fn show_all_windows(&mut self, monitor_rect: &Rect, conn: &Connection, config: &Config) {
        if self.is_visible {
            warn!(
                "show_all_windows was called at workspace {} when it was marked as visible already.",
                self.id
            );
            return;
        }
        self.is_visible = true;
        if self.normal.is_empty() && self.floating.is_empty() && self.docked.is_empty() {
            return;
        }

        let is_below = if !self.normal.is_empty() {
            self.normal.index_rect(0).y > monitor_rect.y
        } else if !self.docked.is_empty() {
            self.docked.at_rect(0).unwrap().y > monitor_rect.y
        } else {
            self.floating.index_rect(0).y > monitor_rect.y
        };
        let move_y = if is_below {
            -(monitor_rect.height as i32)
        } else {
            monitor_rect.height as i32
        };

        let mut avail_rect = Rect {
            x: monitor_rect.x + config.outer_gap_horiz as i32,
            y: monitor_rect.y + config.outer_gap_vert as i32,
            width: monitor_rect.width - config.outer_gap_horiz * 2,
            height: monitor_rect.height - config.outer_gap_vert * 2,
        };
        for rect in self.docked.rect_iter_mut() {
            rect.y += move_y;
            let new_rect = avail_rect.new_rect_magnified(rect);
            rect.x = new_rect.x;
            rect.y = new_rect.y;
            rect.width = new_rect.width;
            rect.height = new_rect.height;
            avail_rect = avail_rect.available_rect_after_adding_rect(&new_rect);
        }

        for (window, rect, _) in self.docked.iter() {
            conn.window_configure(*window, rect, config.border_size);
            conn.map_window(*window);
        }
        if self.focused_type == WindowType::Docked && !self.docked.is_empty() {
            let focused_idx = if self.focused_idx < self.docked.len() {
                self.focused_idx
            } else {
                0
            };
            self.set_focused(
                self.docked.at_window(focused_idx).unwrap(),
                self.focused_type,
                conn,
                config,
            );
        }

        for (window, rect, visible) in self.normal.iter_mut() {
            rect.y += move_y;
            conn.window_configure(*window, rect, config.border_size);
            if *visible {
                conn.map_window(*window);
            }
        }
        // self.fix_existing_normal_windows(&avail_rect, conn, config);
        self.fix_windows_visibility(&avail_rect, conn);

        if self.focused_type == WindowType::Normal && !self.normal.is_empty() {
            let focused_idx = if self.focused_idx < self.normal.len() {
                self.focused_idx
            } else {
                0
            };
            self.set_focused_by_index(focused_idx, self.focused_type, conn, config);
        }

        for (window, rect, visible) in self.floating.iter_mut() {
            rect.y += move_y;
            conn.window_configure(*window, rect, config.border_size);
            if *visible {
                conn.map_window(*window);
            }
        }
        if self.focused_type == WindowType::Floating && !self.floating.is_empty() {
            let focused_idx = if self.focused_idx < self.floating.len() {
                self.focused_idx
            } else {
                0
            };
            self.set_focused_by_index(focused_idx, self.focused_type, conn, config);
        }
        self.focused_via_keyboard = true;
        self.reset_window_interaction_state(conn);
    }

    pub fn hide_all_windows(
        &mut self,
        monitor_rect: &Rect,
        conn: &Connection,
        config: &Config,
        hide_below: bool,
    ) {
        if !self.is_visible {
            warn!(
                "hide_all_windows was called at workspace {} when it was marked as hidden already.",
                self.id
            );
            return;
        }
        self.is_visible = false;
        let move_y = if hide_below {
            monitor_rect.height as i32
        } else {
            -(monitor_rect.height as i32)
        };
        for (window, rect, visible) in self.normal.iter_mut() {
            rect.y += move_y;
            conn.window_configure(*window, rect, config.border_size);
            if *visible {
                conn.unmap_window(*window);
            }
        }
        for (window, rect, visible) in self.floating.iter_mut() {
            rect.y += move_y;
            conn.window_configure(*window, rect, config.border_size);
            if *visible {
                conn.unmap_window(*window);
            }
        }
        for rect in self.docked.rect_iter_mut() {
            rect.y += move_y;
        }
        for idx in 0..self.docked.len() {
            let window = self.docked.at_window(idx).unwrap();
            let rect = self.docked.at_rect(idx).unwrap();
            conn.window_configure(window, &rect, config.border_size);
            conn.unmap_window(window);
        }
        self.reset_window_interaction_state(conn);
    }

    pub fn pop_focused_window(
        &mut self,
        monitor_rect: &Rect,
        conn: &Connection,
        config: &Config,
    ) -> Option<(xcb_window_t, Rect, WindowType)> {
        let avail_rect = self.available_rectangle(monitor_rect, config);
        match self.focused_type {
            WindowType::Normal if self.focused_idx < self.normal.len() => {
                let (removed_window, removed_window_rect, _) =
                    self.normal.remove_at(self.focused_idx);
                let new_focused_idx = if self.focused_idx > 0 {
                    self.focused_idx - 1
                } else {
                    self.focused_idx
                };
                if new_focused_idx < self.normal.len() {
                    self.set_focused_by_index(new_focused_idx, self.focused_type, conn, config);
                } else if !self.floating.is_empty() {
                    self.set_focused_by_index(0, WindowType::Floating, conn, config);
                }
                self.fix_existing_normal_windows(&avail_rect, conn, config);
                self.focused_via_keyboard = true;
                self.reset_window_interaction_state(conn);

                Some((removed_window, removed_window_rect, WindowType::Normal))
            }
            WindowType::Floating if self.focused_idx < self.floating.len() => {
                let (removed_window, removed_window_rect, _) =
                    self.floating.remove_at(self.focused_idx);
                let new_focused_idx = if self.focused_idx > 0 {
                    self.focused_idx - 1
                } else {
                    self.focused_idx
                };
                if new_focused_idx < self.floating.len() {
                    self.set_focused_by_index(self.focused_idx, self.focused_type, conn, config);
                } else if !self.normal.is_empty() {
                    let mut closest_dist = i32::MAX;
                    let mut closest_idx: usize = self.normal.len();
                    for (idx, rect) in self.normal.rect_iter().enumerate() {
                        if rect.intersects_with(&avail_rect) {
                            let current_dist = rect.distance_between_centers(&avail_rect).abs();
                            if closest_dist > current_dist {
                                closest_dist = current_dist;
                                closest_idx = idx;
                            }
                        }
                    }
                    if closest_idx < self.normal.len() {
                        self.set_focused_by_index(
                            new_focused_idx,
                            WindowType::Normal,
                            conn,
                            config,
                        );
                        self.fix_existing_normal_windows(&avail_rect, conn, config);
                        self.focused_via_keyboard = true;
                    }
                }
                self.reset_window_interaction_state(conn);
                Some((removed_window, removed_window_rect, WindowType::Floating))
            }
            WindowType::Docked => None,
            _ => None,
        }
    }

    pub fn handle_enter_notify(
        &mut self,
        window: xcb_window_t,
        conn: &Connection,
        config: &Config,
    ) {
        if self.focused_via_keyboard {
            match self.focused_type {
                WindowType::Normal => {
                    if let Some(currently_focused) = self.normal.at_window(self.focused_idx) {
                        if currently_focused == window {
                            self.focused_via_keyboard = false;
                        }
                    }
                }
                WindowType::Floating => {
                    if let Some(currently_focused) = self.floating.at_window(self.focused_idx) {
                        if currently_focused == window {
                            self.focused_via_keyboard = false;
                        }
                    }
                }
                WindowType::Docked => {
                    if let Some(currently_focused) = self.docked.at_window(self.focused_idx) {
                        if currently_focused == window {
                            self.focused_via_keyboard = false;
                        }
                    }
                }
            }
            return;
        }

        if let Some(index) = self.floating.index_of(window) {
            // if Some(window) != self.floating.at_window(self.focused_idx) {
            self.set_focused_by_index_window(index, window, WindowType::Floating, conn, config);
            // }
            return;
        }
        if let Some(index) = self.normal.index_of(window) {
            // if Some(window) != self.normal.at_window(self.focused_idx) {
            self.set_focused_by_index_window(index, window, WindowType::Normal, conn, config);
            // }
            return;
        }
        if let Some(index) = self.docked.index_of(window) {
            // if Some(window) != self.docked.at_window(self.focused_idx) {
            self.set_focused_by_index_window(index, window, WindowType::Docked, conn, config);
            // }
            return;
        }
    }

    pub fn handle_focus_in(&self, window: xcb_window_t, conn: &Connection, config: &Config) {
        conn.change_window_attrs(
            window,
            XCB_CW_BORDER_PIXEL,
            config.border_color_active_int.unwrap(),
        );
        // if self.focused_type != WindowType::Floating {
        //     self.raise_all_floating_windows(conn);
        // }
    }

    pub fn reconfigure_windows_based_on_changed_available_rect(
        &mut self,
        conn: &Connection,
        config: &Config,
        monitor_rect: &Rect,
    ) {
        let avail_rect = self.available_rectangle(monitor_rect, config);
        self.normal.rect_iter_mut().for_each(|rect| {
            rect.y = avail_rect.y;
            rect.height = avail_rect.height - config.border_size * 2;
        });
        self.fix_existing_normal_windows(&avail_rect, conn, config);
    }

    pub fn remap_windows_with_upd_config(
        &mut self,
        conn: &Connection,
        old_config: &Config,
        config: &Config,
        monitor_rect: &Rect,
    ) {
        let old_avail_rect = self.available_rectangle(monitor_rect, old_config);
        let avail_rect = self.available_rectangle(monitor_rect, config);
        if old_avail_rect != avail_rect || old_config.border_size != config.border_size {
            self.normal.rect_iter_mut().for_each(|r| {
                r.y = avail_rect.y + config.border_size as i32;
                if r.y + r.height as i32 > avail_rect.y + avail_rect.height as i32 {
                    let delta = r.y + r.height as i32 - (avail_rect.y + avail_rect.height as i32);
                    debug_assert!(delta > 0 && delta < r.height as i32);
                    r.height -= delta as u32;
                }
            });
            self.fix_existing_normal_windows(&avail_rect, conn, config);
        }

        let focused_window_maybe = match self.focused_type {
            WindowType::Normal => self.normal.at_window(self.focused_idx),
            WindowType::Floating => self.floating.at_window(self.focused_idx),
            WindowType::Docked => None,
        };
        if old_config.border_color_active_int != config.border_color_active_int {
            if let Some(focused_window) = focused_window_maybe {
                conn.change_window_attrs(
                    focused_window,
                    XCB_CW_BORDER_PIXEL,
                    config.border_color_active_int.unwrap(),
                );
            }
        }
        if old_config.border_color_inactive_int != config.border_color_inactive_int {
            if let Some(focused_window) = focused_window_maybe {
                self.normal.window_iter().for_each(|w| {
                    if *w != focused_window {
                        conn.change_window_attrs(
                            *w,
                            XCB_CW_BORDER_PIXEL,
                            config.border_color_inactive_int.unwrap(),
                        )
                    }
                });
                self.floating.window_iter().for_each(|w| {
                    if *w != focused_window {
                        conn.change_window_attrs(
                            *w,
                            XCB_CW_BORDER_PIXEL,
                            config.border_color_inactive_int.unwrap(),
                        )
                    }
                });
            } else {
                self.normal.window_iter().for_each(|w| {
                    conn.change_window_attrs(
                        *w,
                        XCB_CW_BORDER_PIXEL,
                        config.border_color_inactive_int.unwrap(),
                    )
                });
                self.floating.window_iter().for_each(|w| {
                    conn.change_window_attrs(
                        *w,
                        XCB_CW_BORDER_PIXEL,
                        config.border_color_inactive_int.unwrap(),
                    )
                });
            }
        }
    }

    pub fn center_focused_window(
        &mut self,
        conn: &Connection,
        config: &Config,
        monitor_rect: &Rect,
    ) {
        let avail_rect = self.available_rectangle(monitor_rect, config);
        let focused_rect_maybe = match self.focused_type {
            WindowType::Normal => self.normal.at_rect(self.focused_idx),
            WindowType::Floating => self.floating.at_rect(self.focused_idx),
            WindowType::Docked => None,
        };
        if let Some(focused_rect) = focused_rect_maybe {
            let avail_center_x = avail_rect.x + (avail_rect.width as i32) / 2;
            let focused_center_x = focused_rect.x + (focused_rect.width as i32) / 2;
            let move_x = avail_center_x - focused_center_x;
            self.normal.iter_mut().for_each(|(window, rect, _)| {
                rect.x += move_x;
                conn.window_configure(*window, rect, config.border_size)
            });
            self.floating.iter_mut().for_each(|(window, rect, _)| {
                rect.x += move_x;
                conn.window_configure(*window, rect, config.border_size)
            });
        }
    }

    pub fn find_window_info_by_xcb_id(&self, window: xcb_window_t) -> Option<(usize, WindowType)> {
        if let Some((index, _)) = self
            .normal
            .window_iter()
            .enumerate()
            .find(|(_, w)| **w == window)
        {
            Some((index, WindowType::Normal))
        } else if let Some((index, _)) = self
            .floating
            .window_iter()
            .enumerate()
            .find(|(_, w)| **w == window)
        {
            Some((index, WindowType::Floating))
        } else {
            None
        }
    }

    pub fn handle_destroy_notify(
        &mut self,
        index: usize,
        window_type: WindowType,
        conn: &Connection,
        config: &Config,
        monitor_rect: &Rect,
    ) {
        match window_type {
            WindowType::Normal => {
                self.normal.remove_at(index);
                let avail_rect = self.available_rectangle(monitor_rect, config);
                self.fix_existing_normal_windows(&avail_rect, conn, config);
            }
            WindowType::Floating => {
                self.floating.remove_at(index);
            }
            WindowType::Docked => {
                self.docked.remove_at(index);
            }
        }
    }

    #[inline]
    fn available_rectangle(&self, monitor_rect: &Rect, config: &Config) -> Rect {
        let mut avail_rect = Rect {
            x: monitor_rect.x + config.outer_gap_horiz as i32,
            y: monitor_rect.y + config.outer_gap_vert as i32,
            width: monitor_rect.width - config.outer_gap_horiz * 2,
            height: monitor_rect.height - config.outer_gap_vert * 2,
        };
        for rect in self.docked.rects() {
            avail_rect = avail_rect.available_rect_after_adding_rect(&rect);
        }
        avail_rect
    }

    fn fix_existing_normal_windows(
        &mut self,
        avail_rect: &Rect,
        conn: &Connection,
        config: &Config,
    ) {
        if self.normal.is_empty() {
            return;
        }
        for idx in 0..self.normal.len() {
            let current_rect = self.normal.index_rect(idx).clone();
            let (expected_rect, move_lhs_by, _) = avail_rect.calc_new_rect_added_after_focused(
                current_rect.width,
                current_rect.height,
                None,
                self.normal.rects_slice(0..idx),
                config.inner_gap + config.border_size * 2,
            );
            self.normal.update_rect_at(idx, expected_rect);
            for rect in self.normal.rects_slice_mut(0..idx) {
                rect.x += move_lhs_by;
            }
        }

        self.normal
            .iter()
            .for_each(|(window, rect, _)| conn.window_configure(*window, rect, config.border_size));

        if self.is_visible {
            self.fix_windows_visibility(avail_rect, conn);
        }
    }

    fn fix_windows_visibility(&mut self, monitor_rect: &Rect, conn: &Connection) {
        for (&mut window, rect, visible) in self.normal.iter_mut() {
            let intersects = rect.intersects_with(monitor_rect);
            if intersects != *visible {
                *visible = intersects;
                if intersects {
                    conn.map_window(window);
                } else {
                    conn.unmap_window(window);
                }
            }
        }
    }
}

#[derive(Debug)]
enum WindowMouseInteraction {
    Move {
        window: xcb_window_t,
        index: usize,
        offset_x: i32,
        offset_y: i32,
    },
    Resize {
        window: xcb_window_t,
        index: usize,
        offset_x: i32,
        offset_y: i32,
    },
}

impl Workspace {
    pub fn handle_button_press(
        &mut self,
        x: i32,
        y: i32,
        window: xcb_window_t,
        state: u16,
        detail: xcb_button_t,
        conn: &Connection,
        config: &Config,
        time: xcb_timestamp_t,
    ) {
        let is_alt_pressed = (state as u32 & XCB_MOD_MASK_1) == XCB_MOD_MASK_1;
        let is_left_button_pressed = detail == 1;
        let is_right_button_pressed = detail == 3;
        trace!(
            "handle_button_press - is_alt_pressed: {}, is_left_button_pressed: {}, is_right_button_pressed: {}",
            is_alt_pressed, is_left_button_pressed, is_right_button_pressed
        );
        let mut handled = false;
        if is_alt_pressed && (is_left_button_pressed || is_right_button_pressed) {
            if let Some((window_type, index)) = {
                if let Some(index) = self.floating.index_of(window) {
                    Some((WindowType::Floating, index))
                } else {
                    None
                }
            } {
                if window_type == WindowType::Floating {
                    if self.focused_type != WindowType::Floating
                        || self.floating.at_window(self.focused_idx).unwrap() != window
                    {
                        self.set_focused(window, window_type, conn, config);
                    }
                    if let Err(err) = conn.grab_pointer(
                        XCB_EVENT_MASK_POINTER_MOTION | XCB_EVENT_MASK_BUTTON_RELEASE,
                        window,
                        window,
                    ) {
                        warn!(
                            "failed to grab pointer to drag window {}, err: {}, is_alt_pressed: {}",
                            window, err, is_alt_pressed
                        );
                    } else {
                        trace!(
                            "start dragging window {}, is_alt_pressed: {}",
                            window, is_alt_pressed
                        );
                        // let window_x = self.floating.index_rect(index).x;
                        // let window_y = self.floating.index_rect(index).y;
                        let offset_x = x;
                        let offset_y = y;
                        if is_left_button_pressed {
                            self.window_mouse_interaction = Some(WindowMouseInteraction::Move {
                                window,
                                index,
                                offset_x,
                                offset_y,
                            })
                        } else {
                            self.window_mouse_interaction = Some(WindowMouseInteraction::Resize {
                                window,
                                index,
                                offset_x,
                                offset_y,
                            })
                        }
                        trace!(
                            "window_mouse_interaction: {:?}",
                            self.window_mouse_interaction
                        );
                        handled = true;
                    }
                    conn.flush();
                } else {
                    trace!("clicked window is not floating window");
                }
            } else {
                trace!("clicked window is not found amongst floating windows");
            }
        }
        if !handled {
            conn.allow_events(time);
        }
    }

    pub fn handle_button_release(&mut self, conn: &Connection) {
        trace!("Button release");
        self.reset_window_interaction_state(conn);
        conn.flush();
    }

    pub fn handle_motion_notify(
        &mut self,
        x: i32,
        y: i32,
        window_id: xcb_window_t,
        state: u32,
        conn: &Connection,
        config: &Config,
        monitor_rect: &Rect,
    ) {
        let is_alt_pressed = (state & XCB_MOD_MASK_1) == XCB_MOD_MASK_1;
        let is_left_button_pressed = (state & XCB_BUTTON_MASK_1) == XCB_BUTTON_MASK_1;
        let is_right_button_pressed = (state & XCB_BUTTON_MASK_3) == XCB_BUTTON_MASK_3;
        trace!(
            "MotionNotify x: {}, y: {}, window: {}, is_alt_pressed: {}, left_button: {}, right_button: {}",
            x, y, window_id, is_alt_pressed, is_left_button_pressed, is_right_button_pressed
        );
        if let Some(interaction) = &self.window_mouse_interaction {
            match interaction {
                WindowMouseInteraction::Move {
                    window,
                    index,
                    offset_x,
                    offset_y,
                } => {
                    if !is_left_button_pressed || *window != window_id {
                        self.reset_window_interaction_state(conn);
                        conn.flush();
                    } else {
                        let avail_rect = self.available_rectangle(monitor_rect, config);
                        let mut new_rect = self.floating.index_rect(*index).clone();
                        new_rect.x += x - offset_x;
                        new_rect.y += y - offset_y;
                        new_rect.x = new_rect.x.clamp(
                            avail_rect.x,
                            avail_rect.x + avail_rect.width as i32 - new_rect.width as i32,
                        );
                        new_rect.y = new_rect.y.clamp(
                            avail_rect.y,
                            avail_rect.y + avail_rect.height as i32 - new_rect.height as i32,
                        );
                        *self.floating.index_rect_mut(*index) = new_rect.clone();
                        conn.window_configure(*window, &new_rect, config.border_size);
                        conn.flush();
                    }
                }
                WindowMouseInteraction::Resize {
                    window,
                    index,
                    offset_x,
                    offset_y,
                } => {
                    if !is_right_button_pressed || *window != window_id {
                        self.reset_window_interaction_state(conn);
                        conn.flush();
                    } else {
                        let avail_rect = self.available_rectangle(monitor_rect, config);
                        let mut new_rect = self.floating.index_rect(*index).clone();

                        let new_width = x;
                        let new_height = y;
                        new_rect.width = new_width
                            .clamp(config.minimum_width_tiling as i32, avail_rect.width as i32)
                            as u32;
                        new_rect.height = new_height.clamp(
                            config.minimum_height_tiling as i32,
                            avail_rect.height as i32,
                        ) as u32;

                        *self.floating.index_rect_mut(*index) = new_rect.clone();
                        conn.window_configure(*window, &new_rect, config.border_size);
                        conn.flush();

                        self.window_mouse_interaction = Some(WindowMouseInteraction::Resize {
                            window: *window,
                            index: *index,
                            offset_x: *offset_x,
                            offset_y: *offset_y,
                        })
                    }
                }
            }
        }
    }

    fn reset_window_interaction_state(&mut self, conn: &Connection) {
        if self.window_mouse_interaction.is_some() {
            conn.ungrab_pointer();
            self.window_mouse_interaction = None;
        }
    }
}

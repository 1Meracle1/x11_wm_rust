use base::Rect;
use log::{trace, warn};
use x11_bindings::{
    bindings::{XCB_CW_BORDER_PIXEL, XCB_CW_EVENT_MASK, XCB_EVENT_MASK_FOCUS_CHANGE, xcb_window_t},
    connection::WindowType,
};

use crate::{config::Config, connection::Connection, window::Window};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Workspace {
    pub id: u32,
    pub normal: Vec<Window>,
    pub floating: Vec<Window>,
    pub docked: Vec<Window>,
    pub focused_idx: usize,
    pub focused_type: WindowType,
}

impl Workspace {
    pub fn new(id: u32) -> Self {
        let mut normal = Vec::new();
        normal.reserve(10);

        let mut floating = Vec::new();
        floating.reserve(3);

        Self {
            id,
            normal,
            floating,
            docked: Vec::new(),
            focused_idx: 0,
            focused_type: WindowType::Normal,
        }
    }

    pub fn handle_new_floating_window(
        &mut self,
        window: xcb_window_t,
        rect_hints: Rect,
        monitor_rect: &Rect,
        conn: &Connection,
        config: &Config,
    ) {
        let window_rect = if rect_hints.x != 0 || rect_hints.y != 0 {
            rect_hints
        } else {
            let center_x = (monitor_rect.x + monitor_rect.width as i32) / 2;
            let center_y = (monitor_rect.y + monitor_rect.height as i32) / 2;
            let rect = if rect_hints.width != 0 && rect_hints.height != 0 {
                Rect {
                    x: center_x - (rect_hints.width / 2) as i32,
                    y: center_y - (rect_hints.height / 2) as i32,
                    width: rect_hints.width,
                    height: rect_hints.height,
                }
            } else {
                let width: u32 = 800;
                let height: u32 = 600;
                Rect {
                    x: center_x - (width / 2) as i32,
                    y: center_y - (height / 2) as i32,
                    width,
                    height,
                }
            };
            // check if there are any floating windows that already have the same upper-left corner's position
            // move them slightly lower and to the right, though not in the case it is lower and/or right-er than accepted
            self.floating.sort_by_key(|e| e.rect.x);
            self.floating.sort_by_key(|e| e.rect.y);
            let mut adjusted_rect = rect.clone();
            for w in &self.floating {
                if w.rect.x == rect.x && w.rect.y == rect.y {
                    adjusted_rect.x += 10;
                    adjusted_rect.y += 10;

                    if adjusted_rect.x
                        + (adjusted_rect.width + config.border_size + config.outer_gap_horiz) as i32
                        >= (monitor_rect.x + monitor_rect.width as i32)
                        || adjusted_rect.y
                            + (adjusted_rect.height + config.border_size + config.outer_gap_vert)
                                as i32
                            >= (monitor_rect.y + monitor_rect.height as i32)
                    {
                        adjusted_rect = rect;
                        break;
                    }
                }
            }
            adjusted_rect
        };
        conn.window_configure(window, &window_rect, config.border_size);
        conn.map_window(window);

        self.floating.push(Window {
            window,
            rect: window_rect,
            visible: true,
        });
        self.set_focused(window, WindowType::Floating, conn, config);
    }

    pub fn handle_new_normal_window(
        &mut self,
        window: xcb_window_t,
        monitor_rect: &Rect,
        conn: &Connection,
        config: &Config,
    ) {
        let avail_rect = Rect {
            x: monitor_rect.x + config.outer_gap_horiz as i32,
            y: monitor_rect.y + config.outer_gap_vert as i32,
            width: monitor_rect.width - config.outer_gap_horiz * 2,
            height: monitor_rect.height - config.outer_gap_vert * 2,
        };
        let window_width = ((avail_rect.x + avail_rect.width as i32) as f64
            * config.default_screen_width_percent_tiling) as u32
            - config.border_size * 2;
        let window_height = avail_rect.height - config.border_size * 2;

        let window_rect = if self.normal.is_empty() {
            Rect {
                x: avail_rect.x + (avail_rect.width as i32 - window_width as i32) / 2,
                y: avail_rect.y,
                width: window_width,
                height: window_height,
            }
        } else {
            let left_most_visible_idx = self.get_left_most_visible_normal_idx(monitor_rect);
            let right_most_visible_idx = self.get_right_most_visible_normal_idx(monitor_rect);
            assert!(left_most_visible_idx <= right_most_visible_idx);

            if self.focused_type == WindowType::Normal && self.focused_idx < self.normal.len() {
                assert!(
                    self.normal
                        .get(self.focused_idx)
                        .unwrap()
                        .rect
                        .intersects_with(monitor_rect)
                );

                for w in self.normal[..=self.focused_idx].iter_mut() {
                    w.rect.x -= (window_width + config.inner_gap) as i32;
                    conn.window_configure(w.window, &w.rect, config.border_size);
                }

                {
                    let focused_normal = &self.normal[self.focused_idx];
                    let free_space_left_side =
                        self.normal[left_most_visible_idx].rect.x - avail_rect.x;
                    let free_space_right_side = if right_most_visible_idx == self.focused_idx {
                        avail_rect.x + avail_rect.width as i32
                            - focused_normal.rect.x
                            - (focused_normal.rect.width + config.inner_gap + window_width) as i32
                    } else {
                        avail_rect.x + avail_rect.width as i32
                            - self.normal[right_most_visible_idx].rect.x
                    };

                    let move_right_x = if free_space_right_side + free_space_left_side >= 0 {
                        (free_space_right_side - free_space_left_side) / 2
                    } else if free_space_left_side >= 0 {
                        -free_space_left_side
                    } else if free_space_right_side >= 0 {
                        free_space_right_side
                    } else {
                        0
                    };

                    if move_right_x != 0 {
                        for w in self.normal.iter_mut() {
                            w.rect.x += move_right_x;
                            conn.window_configure(w.window, &w.rect, config.border_size);
                        }
                    }
                };

                let focused_normal = &self.normal[self.focused_idx];
                Rect {
                    x: focused_normal.rect.x
                        + focused_normal.rect.width as i32
                        + config.inner_gap as i32,
                    y: avail_rect.y,
                    width: window_width,
                    height: window_height,
                }
            } else {
                for w in self.normal[..=right_most_visible_idx].iter_mut() {
                    w.rect.x -= (window_width + config.inner_gap) as i32;
                    conn.window_configure(w.window, &w.rect, config.border_size);
                }

                {
                    let right_most_visible = &self.normal[right_most_visible_idx];
                    let free_space_left_side =
                        self.normal[left_most_visible_idx].rect.x - avail_rect.x;
                    let free_space_right_side = avail_rect.x + avail_rect.width as i32
                        - right_most_visible.rect.x
                        - (right_most_visible.rect.width + config.inner_gap + window_width) as i32;

                    let move_right_x = if free_space_right_side + free_space_left_side >= 0 {
                        (free_space_right_side - free_space_left_side) / 2
                    } else if free_space_left_side >= 0 {
                        -free_space_left_side
                    } else if free_space_right_side >= 0 {
                        free_space_right_side
                    } else {
                        0
                    };

                    if move_right_x != 0 {
                        for w in self.normal.iter_mut() {
                            w.rect.x += move_right_x;
                            conn.window_configure(w.window, &w.rect, config.border_size);
                        }
                    }
                };

                let right_most_visible = &self.normal[right_most_visible_idx];
                Rect {
                    x: right_most_visible.rect.x
                        + right_most_visible.rect.width as i32
                        + config.inner_gap as i32,
                    y: avail_rect.y,
                    width: window_width,
                    height: window_height,
                }
            }
        };
        conn.window_configure(window, &window_rect, config.border_size);

        self.normal.push(Window {
            window,
            rect: window_rect,
            visible: false,
        });
        self.normal.sort_by_key(|w| w.rect.x);

        self.fix_visibility(monitor_rect, conn);

        self.set_focused(window, WindowType::Normal, conn, config);
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
                    self.set_focused(
                        self.normal[self.focused_idx - 1].window,
                        self.focused_type,
                        conn,
                        config,
                    );
                    let avail_rect = Rect {
                        x: monitor_rect.x + config.outer_gap_horiz as i32,
                        y: monitor_rect.y + config.outer_gap_vert as i32,
                        width: monitor_rect.width - config.outer_gap_horiz * 2,
                        height: monitor_rect.height - config.outer_gap_vert * 2,
                    };
                    let move_right_x = {
                        let focused_rect = &self.normal[self.focused_idx].rect;
                        if focused_rect.x < avail_rect.x {
                            avail_rect.x - focused_rect.x
                        } else {
                            0
                        }
                    };
                    if move_right_x > 0 {
                        for w in self.normal.iter_mut() {
                            w.rect.x += move_right_x;
                            conn.window_configure(w.window, &w.rect, config.border_size);
                        }
                        self.fix_visibility(monitor_rect, conn);
                    }
                }
            }
            WindowType::Floating => {
                trace!("change focus window left event received when floating window is focused");
            }
            WindowType::Docked => todo!(),
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
                    self.set_focused(
                        self.normal[self.focused_idx + 1].window,
                        self.focused_type,
                        conn,
                        config,
                    );
                    let avail_rect = Rect {
                        x: monitor_rect.x + config.outer_gap_horiz as i32,
                        y: monitor_rect.y + config.outer_gap_vert as i32,
                        width: monitor_rect.width - config.outer_gap_horiz * 2,
                        height: monitor_rect.height - config.outer_gap_vert * 2,
                    };
                    let move_left_x = {
                        let focused_rect = &self.normal[self.focused_idx].rect;
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
                        for w in self.normal.iter_mut() {
                            w.rect.x -= move_left_x;
                            conn.window_configure(w.window, &w.rect, config.border_size);
                        }
                        self.fix_visibility(monitor_rect, conn);
                    }
                }
            }
            WindowType::Floating => {
                trace!("change focus window left event received when floating window is focused");
            }
            WindowType::Docked => todo!(),
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
                    let lhs_window = self.normal[self.focused_idx - 1].clone();
                    let rhs_window = self.normal[self.focused_idx].clone();
                    self.normal[self.focused_idx - 1].window = rhs_window.window;
                    self.normal[self.focused_idx].window = lhs_window.window;

                    conn.window_configure(rhs_window.window, &lhs_window.rect, config.border_size);
                    conn.window_configure(lhs_window.window, &rhs_window.rect, config.border_size);

                    self.normal.sort_by_key(|w| w.rect.x);
                    // self.set_focused(lhs_window.window, self.focused_type, conn, config);
                    self.focused_idx = self.focused_idx - 1;

                    let avail_rect = Rect {
                        x: monitor_rect.x + config.outer_gap_horiz as i32,
                        y: monitor_rect.y + config.outer_gap_vert as i32,
                        width: monitor_rect.width - config.outer_gap_horiz * 2,
                        height: monitor_rect.height - config.outer_gap_vert * 2,
                    };
                    let move_right_x = {
                        let focused_rect = &self.normal[self.focused_idx].rect;
                        if focused_rect.x < avail_rect.x {
                            avail_rect.x - focused_rect.x
                        } else {
                            0
                        }
                    };
                    if move_right_x > 0 {
                        for w in self.normal.iter_mut() {
                            w.rect.x += move_right_x;
                            conn.window_configure(w.window, &w.rect, config.border_size);
                        }
                        self.fix_visibility(monitor_rect, conn);
                    }
                }
            }
            WindowType::Floating => {
                trace!("move window left event received when floating window is focused");
            }
            WindowType::Docked => todo!(),
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
                    let lhs_window = self.normal[self.focused_idx].clone();
                    let rhs_window = self.normal[self.focused_idx + 1].clone();
                    self.normal[self.focused_idx + 1].window = lhs_window.window;
                    self.normal[self.focused_idx].window = rhs_window.window;

                    conn.window_configure(rhs_window.window, &lhs_window.rect, config.border_size);
                    conn.window_configure(lhs_window.window, &rhs_window.rect, config.border_size);

                    self.normal.sort_by_key(|w| w.rect.x);
                    // self.set_focused(rhs_window.window, self.focused_type, conn, config);
                    self.focused_idx = self.focused_idx + 1;

                    let avail_rect = Rect {
                        x: monitor_rect.x + config.outer_gap_horiz as i32,
                        y: monitor_rect.y + config.outer_gap_vert as i32,
                        width: monitor_rect.width - config.outer_gap_horiz * 2,
                        height: monitor_rect.height - config.outer_gap_vert * 2,
                    };
                    let move_left_x = {
                        let focused_rect = &self.normal[self.focused_idx].rect;
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
                        for w in self.normal.iter_mut() {
                            w.rect.x -= move_left_x;
                            conn.window_configure(w.window, &w.rect, config.border_size);
                        }
                        self.fix_visibility(monitor_rect, conn);
                    }
                }
            }
            WindowType::Floating => {
                trace!("move window left event received when floating window is focused");
            }
            WindowType::Docked => todo!(),
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
                WindowType::Normal => {
                    let (idx, _) = self
                        .normal
                        .iter()
                        .enumerate()
                        .find(|(_, w)| w.window == window)
                        .unwrap();
                    idx
                }
                WindowType::Floating => {
                    let (idx, _) = self
                        .floating
                        .iter()
                        .enumerate()
                        .find(|(_, w)| w.window == window)
                        .unwrap();
                    idx
                }
                WindowType::Docked => {
                    let (idx, _) = self
                        .docked
                        .iter()
                        .enumerate()
                        .find(|(_, w)| w.window == window)
                        .unwrap();
                    idx
                }
            }
        };

        self.focused_idx = index;
        self.focused_type = window_type;

        conn.change_window_attrs(
            window,
            XCB_CW_BORDER_PIXEL,
            config.border_color_active_int.unwrap(),
        );
        conn.change_window_attrs(window, XCB_CW_EVENT_MASK, XCB_EVENT_MASK_FOCUS_CHANGE);
        trace!("apply focus to {}", window);
        conn.window_set_input_focus(window);
    }

    pub fn handle_resize_window_horizontal(
        &mut self,
        conn: &Connection,
        config: &Config,
        monitor_rect: &Rect,
        size_change_pixels: i32,
    ) {
        let avail_rect = Rect {
            x: monitor_rect.x + config.outer_gap_horiz as i32,
            y: monitor_rect.y + config.outer_gap_vert as i32,
            width: monitor_rect.width - config.outer_gap_horiz * 2,
            height: monitor_rect.height - config.outer_gap_vert * 2,
        };
        match self.focused_type {
            WindowType::Normal if self.focused_idx < self.normal.len() => {
                let focused_rect = self.normal[self.focused_idx].rect.clone();
                let new_width = (focused_rect.width as i32 + size_change_pixels)
                    .clamp(config.minimum_width_tiling as i32, avail_rect.width as i32);
                let new_x = (focused_rect.x - size_change_pixels / 2)
                    .clamp(avail_rect.x, avail_rect.x + avail_rect.width as i32);

                self.normal[self.focused_idx].rect.width = new_width as u32;
                self.normal[self.focused_idx].rect.x = new_x;

                let move_x_from_left = new_x - focused_rect.x;
                let move_x_from_right = move_x_from_left + new_width - focused_rect.width as i32;
                if move_x_from_left != 0 {
                    for w in self.normal[..self.focused_idx].iter_mut() {
                        w.rect.x += move_x_from_left;
                    }
                }
                if move_x_from_right != 0 {
                    for w in self.normal[self.focused_idx + 1..].iter_mut() {
                        w.rect.x += move_x_from_right;
                    }
                }
                for w in self.normal.iter() {
                    conn.window_configure(w.window, &w.rect, config.border_size);
                }

                self.fix_visibility(monitor_rect, conn);
            }
            WindowType::Floating => {
                trace!("horizontal resize window event received when floating window is focused");
            }
            WindowType::Docked => todo!(),
            _ => {
                warn!("horizontal resize window when docked window is focused");
            }
        }
    }

    fn get_left_most_visible_normal_idx(&self, monitor_rect: &Rect) -> usize {
        let mut res = self.normal.len();
        let mut start_x = monitor_rect.x + monitor_rect.width as i32;
        for idx in 0..self.normal.len() {
            let w = &self.normal[idx];
            if w.rect.intersects_with(monitor_rect) && w.rect.x < start_x {
                start_x = w.rect.x;
                res = idx;
            }
        }
        res
    }

    fn get_right_most_visible_normal_idx(&self, monitor_rect: &Rect) -> usize {
        let mut res = 0;
        let mut start_x = monitor_rect.x;
        for idx in 0..self.normal.len() {
            let w = &self.normal[idx];
            if w.rect.intersects_with(monitor_rect) && w.rect.x > start_x {
                start_x = w.rect.x;
                res = idx;
            }
        }
        res
    }

    fn fix_visibility(&mut self, monitor_rect: &Rect, conn: &Connection) {
        for w in self.normal.iter_mut() {
            if w.rect.intersects_with(monitor_rect) {
                if !w.visible {
                    w.visible = true;
                    conn.map_window(w.window);
                }
            } else {
                if w.visible {
                    w.visible = false;
                    conn.unmap_window(w.window);
                }
            }
        }
    }
}

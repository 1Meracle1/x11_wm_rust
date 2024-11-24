use crate::{
    config::Config,
    monitor::Rect,
    window::{Window, WindowType},
};
use log::{debug, error};
use xcb::x;

#[derive(Debug)]
pub struct Workspace {
    tiling_windows: Vec<Window>,
    floating_windows: Vec<Window>,
    available_area: Rect,
    docked_windows: Vec<Window>,
    focused_window: Option<x::Window>,
    focused_window_type: WindowType,
    // config fields
    outer_gap_horiz: u16,
    outer_gap_vert: u16,
    inner_gap: u16,
}

impl Workspace {
    pub fn new(available_area: Rect, config: &Config, conn: &xcb::Connection) -> Self {
        let mut tiling_windows = Vec::new();
        tiling_windows.reserve(3);

        let mut res = Self {
            tiling_windows,
            floating_windows: Vec::new(),
            available_area,
            docked_windows: Vec::new(),
            focused_window: None,
            focused_window_type: WindowType::default(),
            outer_gap_horiz: 0,
            outer_gap_vert: 0,
            inner_gap: config.workspaces.inner_gap,
        };
        res.update_gaps(config, conn);

        res
    }

    pub fn update_gaps(&mut self, config: &Config, conn: &xcb::Connection) {
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
        // let diff_width = self.available_area.width as i16 - new_available_area.width as i16;
        // let diff_height = self.available_area.height as i16 - new_available_area.height as i16;
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
                item.show(conn);
            }
        });
    }

    pub fn hide_windows_out_available_area(&mut self, conn: &xcb::Connection) {
        self.tiling_windows.iter_mut().for_each(|item| {
            let intersects = item.intersects_with_rect(&self.available_area);
            if !intersects && item.mapped {
                item.hide(conn);
            }
        });
    }

    pub fn make_space_for_tiling_window(&mut self, width: u16, conn: &xcb::Connection) -> Rect {
        let rect = if self.tiling_windows.is_empty() {
            Rect {
                x: ((self.available_area.x + self.available_area.width as i16) / 2)
                    - (width as i16) / 2,
                y: self.available_area.y,
                width,
                height: self.available_area.height,
            }
        } else if self.focused_window.is_none() || self.focused_window_type != WindowType::Tiling {
            let last_window_rect = &self.tiling_windows.last().unwrap().rect;
            let diff_last_vs_avail = last_window_rect.x + last_window_rect.width as i16
                - self.available_area.x
                - self.available_area.width as i16;
            if diff_last_vs_avail > 0 {
                self.tiling_windows.iter_mut().for_each(|item| {
                    item.rect.x -= diff_last_vs_avail;
                })
            }
            let last_window_rect = &self.tiling_windows.last().unwrap().rect;
            Rect {
                x: last_window_rect.x + self.inner_gap as i16,
                y: self.available_area.y,
                width,
                height: self.available_area.height,
            }
        } else {
            let focused_xcb_window = self.focused_window.unwrap();
            if let Some((index, focused)) = self
                .tiling_windows
                .iter_mut()
                .enumerate()
                .find(|(_, item)| item.xcb_window == focused_xcb_window)
            {
                let diff_focus_vs_avail = focused.rect.x + focused.rect.width as i16
                    - self.available_area.x
                    - self.available_area.width as i16;
                if diff_focus_vs_avail > 0 {
                    self.tiling_windows[..=index].iter_mut().for_each(|item| {
                        item.rect.x -= diff_focus_vs_avail;
                    });
                }
                self.tiling_windows[index + 1..]
                    .iter_mut()
                    .for_each(|item| {
                        item.rect.x += width as i16 - diff_focus_vs_avail;
                    });

                Rect {
                    x: self.tiling_windows[index].rect.x + self.inner_gap as i16,
                    y: self.available_area.y,
                    width,
                    height: self.available_area.height,
                }
            } else {
                let last_window_rect = &self.tiling_windows.last().unwrap().rect;
                let diff_last_vs_avail = last_window_rect.x + last_window_rect.width as i16
                    - self.available_area.x
                    - self.available_area.width as i16;
                if diff_last_vs_avail > 0 {
                    self.tiling_windows.iter_mut().for_each(|item| {
                        item.rect.x -= diff_last_vs_avail;
                    })
                }
                let last_window_rect = &self.tiling_windows.last().unwrap().rect;
                Rect {
                    x: last_window_rect.x + self.inner_gap as i16,
                    y: self.available_area.y,
                    width,
                    height: self.available_area.height,
                }
            }
        };
        debug!("window rect: {:#?}", rect);
        debug!("available rect: {:#?}", self.available_area);

        self.tiling_windows
            .iter()
            .for_each(|item| item.configure(conn));
        self.show_windows_in_available_area(conn);
        self.hide_windows_out_available_area(conn);

        rect
    }

    pub fn set_focused(&mut self, window: &Window, conn: &xcb::Connection, config: &Config) {
        if let Some(focused_id) = self.focused_window {
            if let Some(focused_window) = self
                .tiling_windows
                .iter()
                .find(|item| item.xcb_window == focused_id)
            {
                focused_window
                    .change_border_color(conn, config.window.border.color_inactive_u32.unwrap());
            }
        }
        self.focused_window_type = window.r#type;
        self.focused_window = Some(window.xcb_window);
        window.change_border_color(conn, config.window.border.color_active_u32.unwrap());
    }

    pub fn add_window(&mut self, window: Window) {
        match window.r#type {
            WindowType::Tiling => {
                self.tiling_windows.push(window);
            }
            WindowType::Floating => {
                self.floating_windows.push(window);
            }
            WindowType::Docking => {
                self.docked_windows.push(window);
            }
        };
        self.sort_windows();
    }

    fn sort_windows(&mut self) {
        self.tiling_windows.sort();
        self.floating_windows.sort();
    }
}

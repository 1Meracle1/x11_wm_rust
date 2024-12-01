use log::debug;
use xcb::x;

use crate::monitor::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowType {
    Tiling,
    Floating,
    Docking,
}

impl Default for WindowType {
    fn default() -> Self {
        WindowType::Tiling
    }
}

#[derive(Debug)]
pub struct Window {
    pub rect: Rect,
    pub id: x::Window,
    pub r#type: WindowType,
    pub mapped: bool,
    pub border_size: u16,
}

impl Window {
    pub fn new(rect: Rect, xcb_window: x::Window, r#type: WindowType, border_size: u16) -> Self {
        Self {
            rect,
            id: xcb_window,
            r#type,
            mapped: false,
            border_size,
        }
    }

    #[inline]
    pub fn map(&mut self, conn: &xcb::Connection) {
        conn.send_request(&x::MapWindow { window: self.id });
        self.mapped = true;
    }

    #[inline]
    pub fn unmap(&mut self, conn: &xcb::Connection) {
        conn.send_request(&x::UnmapWindow { window: self.id });
        self.mapped = false;
    }

    #[inline]
    pub fn show(&mut self, conn: &xcb::Connection) {
        conn.send_request(&x::MapWindow { window: self.id });
    }

    #[inline]
    pub fn hide(&mut self, conn: &xcb::Connection) {
        conn.send_request(&x::UnmapWindow { window: self.id });
    }

    pub fn subscribe_to_wm_events(&self, conn: &xcb::Connection) {
        conn.send_request(&x::ChangeWindowAttributes {
            window: self.id,
            value_list: &[x::Cw::EventMask(
                x::EventMask::PROPERTY_CHANGE
                    | x::EventMask::FOCUS_CHANGE
                    | x::EventMask::ENTER_WINDOW
                    | x::EventMask::LEAVE_WINDOW,
            )],
        });
    }

    #[inline]
    pub fn configure(&self, conn: &xcb::Connection) {
        conn.send_request(&x::ConfigureWindow {
            window: self.id,
            value_list: &[
                x::ConfigWindow::X(self.rect.x as i32),
                x::ConfigWindow::Y(self.rect.y as i32),
                x::ConfigWindow::Width(self.rect.width as u32),
                x::ConfigWindow::Height(self.rect.height as u32),
                x::ConfigWindow::BorderWidth(self.border_size as u32),
            ],
        });
    }

    #[inline]
    pub fn change_border_color(&self, conn: &xcb::Connection, color: u32) {
        debug!("change border color: {} for {:?}", color, self.id);
        conn.send_request(&x::ChangeWindowAttributes {
            window: self.id,
            value_list: &[x::Cw::BorderPixel(color)],
        });
    }

    #[inline]
    pub fn set_input_focus(&self, conn: &xcb::Connection) {
        debug!("set input focus for {:?}", self.id);
        conn.send_request(&x::SetInputFocus {
            revert_to: x::InputFocus::PointerRoot,
            focus: self.id,
            time: x::CURRENT_TIME,
        });
    }

    #[inline]
    pub fn intersects_with(&self, x: i16, y: i16, width: u16, height: u16) -> bool {
        if self.rect.x < x && self.rect.x + (self.rect.width as i16) < x
            || self.rect.x > x + (width as i16)
        {
            return false;
        }
        if self.rect.y < y && self.rect.y + (self.rect.height as i16) < y
            || self.rect.y > y + (height as i16)
        {
            return false;
        }
        true
    }

    #[inline]
    pub fn intersects_with_rect(&self, rect: &Rect) -> bool {
        self.intersects_with(rect.x, rect.y, rect.width, rect.height)
    }

    #[inline]
    pub fn point_belongs_to(&self, pos_x: i16, pos_y: i16) -> bool {
        if pos_x < self.rect.x || pos_x > self.rect.x + self.rect.width as i16 {
            return false;
        }
        if pos_y < self.rect.y || pos_y > self.rect.y + self.rect.height as i16 {
            return false;
        }
        true
    }
}

impl PartialEq for Window {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Window {}

impl PartialOrd for Window {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.rect.x.partial_cmp(&other.rect.x)
    }
}

impl Ord for Window {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rect.x.cmp(&other.rect.x)
    }
}

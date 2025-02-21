use base::Rect;
use x11_bindings::bindings::xcb_window_t;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Window {
    pub window: xcb_window_t,
    pub rect: Rect,
    pub visible: bool,
}

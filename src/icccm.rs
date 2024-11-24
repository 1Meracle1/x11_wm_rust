use xcb::x;

pub struct Icccm;

impl Icccm {
    pub fn new() -> Self {
        Self {}
    }

    pub fn set_wm_class(conn: &xcb::Connection, window: x::Window, class: &str) {
        conn.send_request(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window,
            property: x::ATOM_WM_CLASS,
            r#type: x::ATOM_STRING,
            data: class.as_bytes(),
        });
    }

    pub fn set_wm_name(conn: &xcb::Connection, window: x::Window, name: &str) {
        conn.send_request(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window,
            property: x::ATOM_WM_NAME,
            r#type: x::ATOM_STRING,
            data: name.as_bytes(),
        });
    }
}

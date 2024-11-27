use std::io::Read;

use log::error;
use xcb::x::{self};

use crate::window::WindowType;

xcb::atoms_struct! {
    #[derive(Debug)]
    struct NetAtoms {
        supported => b"_NET_SUPPORTED",
        supporing_wm_check => b"_NET_SUPPORTING_WM_CHECK",
        desktop_names => b"_NET_DESKTOP_NAMES",
        desktop_viewport => b"_NET_DESKTOP_VIEWPORT",
        number_of_desktops => b"_NET_NUMBER_OF_DESKTOPS",
        current_desktop => b"_NET_CURRENT_DESKTOP",
        client_list => b"_NET_CLIENT_LIST",
        active_window => b"_NET_ACTIVE_WINDOW",
        wm_name => b"_NET_WM_NAME",
        close_window => b"_NET_CLOSE_WINDOW",
        strut_partial => b"_NET_WM_STRUT_PARTIAL",
        wm_desktop => b"_NET_WM_DESKTOP",
        wm_state => b"_NET_WM_STATE",
        wm_state_hidden => b"_NET_WM_STATE_HIDDEN",
        wm_state_fullscreen => b"_NET_WM_STATE_FULLSCREEN",
        wm_state_below => b"_NET_WM_STATE_BELOW",
        wm_state_above => b"_NET_WM_STATE_ABOVE",
        wm_state_sticky => b"_NET_WM_STATE_STICKY",
        wm_state_demands_attention => b"_NET_WM_STATE_DEMANDS_ATTENTION",
        wm_window_type => b"_NET_WM_WINDOW_TYPE",
        wm_window_type_normal => b"_NET_WM_WINDOW_TYPE_NORMAL",
        wm_window_type_dock => b"_NET_WM_WINDOW_TYPE_DOCK",
        wm_window_type_desktop => b"_NET_WM_WINDOW_TYPE_DESKTOP",
        wm_window_type_notification => b"_NET_WM_WINDOW_TYPE_NOTIFICATION",
        wm_window_type_dialog => b"_NET_WM_WINDOW_TYPE_DIALOG",
        wm_window_type_splash => b"_NET_WM_WINDOW_TYPE_SPLASH",
        wm_window_type_utility => b"_NET_WM_WINDOW_TYPE_UTILITY",
        wm_window_type_toolbar => b"_NET_WM_WINDOW_TYPE_TOOLBAR",
    }
}

#[derive(Debug)]
pub struct StrutPartial {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
    pub left_start_y: u32,
    pub left_end_y: u32,
    pub right_start_y: u32,
    pub right_end_y: u32,
    pub top_start_x: u32,
    pub top_end_x: u32,
    pub bottom_start_x: u32,
    pub bottom_end_x: u32,
}

pub struct Ewmh {
    atoms: NetAtoms,
}

impl Ewmh {
    pub fn new(conn: &xcb::Connection, root_window: x::Window) -> Self {
        let ewmh = Self {
            atoms: NetAtoms::intern_all(conn).unwrap(),
        };
        ewmh.set_net_supported(conn, root_window);

        ewmh
    }

    pub fn set_net_supported(&self, conn: &xcb::Connection, window: x::Window) {
        conn.send_request(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window,
            property: self.atoms.supported,
            r#type: x::ATOM_ATOM,
            data: &[
                self.atoms.supporing_wm_check,
                self.atoms.desktop_names,
                self.atoms.desktop_viewport,
                self.atoms.number_of_desktops,
                self.atoms.current_desktop,
                self.atoms.client_list,
                self.atoms.active_window,
                self.atoms.wm_name,
                self.atoms.close_window,
                self.atoms.strut_partial,
                self.atoms.wm_desktop,
                self.atoms.wm_state,
                self.atoms.wm_state_hidden,
                self.atoms.wm_state_fullscreen,
                self.atoms.wm_state_below,
                self.atoms.wm_state_above,
                self.atoms.wm_state_sticky,
                self.atoms.wm_state_demands_attention,
                self.atoms.wm_window_type,
                self.atoms.wm_window_type_dock,
                self.atoms.wm_window_type_desktop,
                self.atoms.wm_window_type_notification,
                self.atoms.wm_window_type_dialog,
                self.atoms.wm_window_type_splash,
                self.atoms.wm_window_type_utility,
                self.atoms.wm_window_type_toolbar,
            ],
        });
    }

    pub fn update_current_desktop(
        &self,
        conn: &xcb::Connection,
        root_window: x::Window,
        new_index: u32,
    ) {
        let event = x::ClientMessageEvent::new(
            root_window,
            self.atoms.current_desktop,
            x::ClientMessageData::Data32([new_index, x::CURRENT_TIME, 0, 0, 0]),
        );
        conn.send_request(&x::SendEvent {
            propagate: false,
            destination: x::SendEventDest::Window(root_window),
            event_mask: x::EventMask::SUBSTRUCTURE_NOTIFY | x::EventMask::SUBSTRUCTURE_REDIRECT,
            event: &event,
        });
    }

    pub fn update_number_of_desktops(
        &self,
        conn: &xcb::Connection,
        root_window: x::Window,
        new_num_desktops: u32,
    ) {
        let event = x::ClientMessageEvent::new(
            root_window,
            self.atoms.number_of_desktops,
            x::ClientMessageData::Data32([new_num_desktops, 0, 0, 0, 0]),
        );
        conn.send_request(&x::SendEvent {
            propagate: false,
            destination: x::SendEventDest::Window(root_window),
            event_mask: x::EventMask::SUBSTRUCTURE_NOTIFY | x::EventMask::SUBSTRUCTURE_REDIRECT,
            event: &event,
        });
    }

    pub fn update_window_wm_desktop(
        &self,
        conn: &xcb::Connection,
        window: x::Window,
        new_desktop: u32,
    ) {
        let event = x::ClientMessageEvent::new(
            window,
            self.atoms.wm_desktop,
            x::ClientMessageData::Data32([new_desktop, 1, 0, 0, 0]),
        );
        conn.send_request(&x::SendEvent {
            propagate: false,
            destination: x::SendEventDest::Window(window),
            event_mask: x::EventMask::SUBSTRUCTURE_NOTIFY | x::EventMask::SUBSTRUCTURE_REDIRECT,
            event: &event,
        });
    }

    pub fn get_window_type(&self, conn: &xcb::Connection, window: x::Window) -> WindowType {
        let cookie = conn.send_request(&x::GetProperty {
            delete: false,
            window,
            property: self.atoms.wm_window_type,
            r#type: x::ATOM_ATOM,
            long_offset: 0,
            long_length: 1,
        });
        match conn.wait_for_reply(cookie) {
            Ok(reply) => {
                let wm_type = reply.r#type();
                match wm_type {
                    _ if wm_type == self.atoms.wm_window_type_dock => WindowType::Docking,
                    _ if wm_type == self.atoms.wm_window_type_desktop => WindowType::Floating,
                    _ if wm_type == self.atoms.wm_window_type_notification => WindowType::Floating,
                    _ if wm_type == self.atoms.wm_window_type_dialog => WindowType::Floating,
                    _ if wm_type == self.atoms.wm_window_type_splash => WindowType::Floating,
                    _ if wm_type == self.atoms.wm_window_type_utility => WindowType::Floating,
                    _ if wm_type == self.atoms.wm_window_type_toolbar => WindowType::Floating,
                    _ => WindowType::Tiling,
                }
            }
            Err(_) => WindowType::Tiling,
        }
    }

    pub fn get_strut_partial(
        &self,
        conn: &xcb::Connection,
        window: x::Window,
    ) -> Option<StrutPartial> {
        let cookie = conn.send_request(&x::GetProperty {
            delete: false,
            window,
            property: self.atoms.strut_partial,
            r#type: x::ATOM_CARDINAL,
            long_offset: 0,
            long_length: 12,
        });
        match conn.wait_for_reply(cookie) {
            Ok(reply) => {
                let values = reply.value::<u32>();
                Some(StrutPartial {
                    left: values[0],
                    right: values[1],
                    top: values[2],
                    bottom: values[3],
                    left_start_y: values[4],
                    left_end_y: values[5],
                    right_start_y: values[6],
                    right_end_y: values[7],
                    top_start_x: values[8],
                    top_end_x: values[9],
                    bottom_start_x: values[10],
                    bottom_end_x: values[11],
                })
            }
            Err(_) => None,
        }
    }

    pub fn get_desktop_index(&self, conn: &xcb::Connection, window: x::Window) -> Option<u32> {
        let cookie = conn.send_request(&x::GetProperty {
            delete: false,
            window,
            property: self.atoms.wm_desktop,
            r#type: x::ATOM_CARDINAL,
            long_offset: 0,
            long_length: 1,
        });
        match conn.wait_for_reply(cookie) {
            Ok(reply) => bytes_to_u32(reply.value()),
            Err(err) => {
                error!(
                    "Failed to fetch desktop index for window {:?}, error: {}",
                    window, err
                );
                None
            }
        }
    }
}

fn bytes_to_u32(bytes: &[u8]) -> Option<u32> {
    let mut padded = [0u8; 4];
    let len = bytes.len().min(4);
    padded[..len].copy_from_slice(&bytes[..len]);
    Some(u32::from_le_bytes(padded))
}

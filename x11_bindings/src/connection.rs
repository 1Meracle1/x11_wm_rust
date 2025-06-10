use std::{collections::HashMap, ffi::CString, mem::MaybeUninit, os::fd::RawFd};

use base::Rect;

use crate::bindings::{
    xcb_allow_events, xcb_allow_t, xcb_arc_t, xcb_atom_t, xcb_button_press_event_t, xcb_button_release_event_t, xcb_button_t, xcb_change_gc, xcb_change_property, xcb_change_window_attributes, xcb_change_window_attributes_checked, xcb_client_message_data_t, xcb_client_message_event_t, xcb_configure_window, xcb_configure_window_checked, xcb_connection_has_error, xcb_connection_t, xcb_create_cursor, xcb_create_cursor_checked, xcb_create_gc, xcb_create_gc_checked, xcb_create_pixmap, xcb_create_pixmap_checked, xcb_create_window, xcb_cursor_context_free, xcb_cursor_context_new, xcb_cursor_context_t, xcb_cursor_load_cursor, xcb_cursor_t, xcb_cw_t, xcb_destroy_notify_event_t, xcb_destroy_window, xcb_disconnect, xcb_enter_notify_event_t, xcb_event_mask_t, xcb_ewmh_connection_t, xcb_ewmh_get_atoms_reply_t, xcb_ewmh_get_atoms_reply_wipe, xcb_ewmh_get_cardinal_reply, xcb_ewmh_get_wm_desktop, xcb_ewmh_get_wm_strut_partial, xcb_ewmh_get_wm_strut_partial_reply, xcb_ewmh_get_wm_window_type, xcb_ewmh_get_wm_window_type_reply, xcb_ewmh_init_atoms, xcb_ewmh_init_atoms_replies, xcb_ewmh_set_supported_checked, xcb_ewmh_wm_strut_partial_t, xcb_flush, xcb_focus_in_event_t, xcb_focus_out_event_t, xcb_free_gc, xcb_free_pixmap, xcb_gc_t, xcb_gcontext_t, xcb_generate_id, xcb_generic_error_t, xcb_generic_event_t, xcb_get_file_descriptor, xcb_get_property, xcb_get_property_reply, xcb_get_property_value, xcb_get_property_value_length, xcb_get_setup, xcb_get_window_attributes, xcb_get_window_attributes_reply, xcb_grab_button, xcb_grab_key, xcb_grab_pointer, xcb_grab_pointer_reply, xcb_icccm_set_wm_normal_hints, xcb_image_create, xcb_image_create_native, xcb_image_destroy, xcb_image_put, xcb_intern_atom, xcb_intern_atom_cookie_t, xcb_intern_atom_reply, xcb_key_press_event_t, xcb_keycode_t, xcb_leave_notify_event_t, xcb_map_request_event_t, xcb_map_window, xcb_mod_mask_t, xcb_motion_notify_event_t, xcb_notify_mode_t, xcb_pixmap_t, xcb_point_t, xcb_poll_for_event, xcb_poly_fill_arc, xcb_poly_fill_rectangle, xcb_poly_point, xcb_put_image, xcb_query_extension, xcb_query_extension_reply, xcb_randr_notify_event_t, xcb_randr_screen_change_notify_event_t, xcb_randr_select_input_checked, xcb_rectangle_t, xcb_request_check, xcb_screen_t, xcb_send_event, xcb_set_input_focus, xcb_setup_roots_iterator, xcb_shape_mask, xcb_size_hints_t, xcb_ungrab_key, xcb_ungrab_pointer, xcb_unmap_notify_event_t, xcb_unmap_window, xcb_wait_for_event, xcb_window_t, xcb_xkb_select_events, xcb_xkb_state_notify_event_t, xkb_context, xkb_context_new, xkb_keymap, xkb_keymap_get_as_string, xkb_state, xkb_state_update_mask, xkb_x11_get_core_keyboard_device_id, xkb_x11_keymap_new_from_device, xkb_x11_setup_xkb_extension, xkb_x11_state_new_from_device, XCloseDisplay, XDefaultRootWindow, XDefineCursor, XDisplay, XFree, XGetAtomName, XGetXCBConnection, XOpenDisplay, XcursorFilenameLoadCursor, XkbAllControlsMaskConst, XkbAllocKeyboard, XkbFreeKeyboard, XkbGetControls, XkbGetNames, XkbGroupNamesMask, XkbSymbolsNameMask, XCB_ACCESS, XCB_ALLOC, XCB_ATOM, XCB_ATOM_ATOM, XCB_ATOM_STRING, XCB_ATOM_WM_CLASS, XCB_ATOM_WM_NORMAL_HINTS, XCB_BUTTON_PRESS, XCB_BUTTON_RELEASE, XCB_CLIENT_MESSAGE, XCB_COLORMAP, XCB_CONFIG_WINDOW_BORDER_WIDTH, XCB_CONFIG_WINDOW_HEIGHT, XCB_CONFIG_WINDOW_STACK_MODE, XCB_CONFIG_WINDOW_WIDTH, XCB_CONFIG_WINDOW_X, XCB_CONFIG_WINDOW_Y, XCB_COORD_MODE_ORIGIN, XCB_COPY_FROM_PARENT, XCB_CURRENT_TIME, XCB_CURSOR, XCB_CW_CURSOR, XCB_DESTROY_NOTIFY, XCB_DRAWABLE, XCB_ENTER_NOTIFY, XCB_EVENT_MASK_BUTTON_PRESS, XCB_EVENT_MASK_NO_EVENT, XCB_FOCUS_IN, XCB_FOCUS_OUT, XCB_FONT, XCB_GET_PROPERTY_TYPE_ANY, XCB_GRAB_MODE_ASYNC, XCB_G_CONTEXT, XCB_ID_CHOICE, XCB_IMAGE_FORMAT_XY_PIXMAP, XCB_IMAGE_FORMAT_Z_PIXMAP, XCB_IMAGE_ORDER_LSB_FIRST, XCB_IMPLEMENTATION, XCB_INPUT_FOCUS_POINTER_ROOT, XCB_KEY_PRESS, XCB_LEAVE_NOTIFY, XCB_LENGTH, XCB_MAP_REQUEST, XCB_MATCH, XCB_MOD_MASK_ANY, XCB_MOTION_NOTIFY, XCB_NAME, XCB_NONE, XCB_PIXMAP, XCB_PROP_MODE_REPLACE, XCB_RANDR_NOTIFY, XCB_RANDR_NOTIFY_MASK_SCREEN_CHANGE, XCB_RANDR_SCREEN_CHANGE_NOTIFY, XCB_SHAPE_SK_BOUNDING, XCB_SHAPE_SO_SET, XCB_STACK_MODE_ABOVE, XCB_UNMAP_NOTIFY, XCB_WINDOW, XCB_WINDOW_CLASS_INPUT_OUTPUT, XCB_XKB_EVENT_TYPE_CONTROLS_NOTIFY, XCB_XKB_EVENT_TYPE_NAMES_NOTIFY, XCB_XKB_EVENT_TYPE_STATE_NOTIFY, XCB_XKB_STATE_NOTIFY, XKB_CONTEXT_NO_FLAGS, XKB_KEYMAP_COMPILE_NO_FLAGS, XKB_KEYMAP_FORMAT_TEXT_V1, XKB_X11_MIN_MAJOR_XKB_VERSION, XKB_X11_MIN_MINOR_XKB_VERSION, XKB_X11_SETUP_XKB_EXTENSION_NO_FLAGS
};

#[derive(Debug)]
pub enum ConnectionError {
    UnableToOpenXDisplay,
    UnableToGetXCBConnection,
    UnableToInitEwmh,
    UnableToSetSupportedEwmhAtoms,
    UnableToChangeWindowAttrs((xcb_window_t, String)),
    UnableToGrabPointer((xcb_window_t, String)),
    UnableToQueryRandrExtension(String),
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionError::UnableToOpenXDisplay => write!(f, "Unable to open XDisplay"),
            ConnectionError::UnableToGetXCBConnection => {
                write!(f, "Unable to get XCB Connection from XDisplay")
            }
            ConnectionError::UnableToChangeWindowAttrs((window, err)) => write!(
                f,
                "Unable to change attributes for window {}, error {}",
                window, err
            ),
            ConnectionError::UnableToInitEwmh => write!(f, "Unable to initialize EWMH atoms"),
            ConnectionError::UnableToSetSupportedEwmhAtoms => {
                write!(f, "Unable to set supported EWMH atoms")
            }
            ConnectionError::UnableToGrabPointer((window, err)) => write!(
                f,
                "Unable to grab pointer for window {}, error {}",
                window, err
            ),
            ConnectionError::UnableToQueryRandrExtension(err) => {
                write!(f, "Unable to query randr extension, error {}", err)
            }
        }
    }
}

#[allow(dead_code)]
pub struct Connection {
    display: *mut XDisplay,
    conn: *mut xcb_connection_t,
    screen: *mut xcb_screen_t,
    ewmh: *mut xcb_ewmh_connection_t,
    randr_event_base: u8,
    xkb_event_base: u8,
    xkb_device_id: i32,
    xkb_context: *mut xkb_context,
    xkb_keymap: *mut xkb_keymap,
    xkb_state: *mut xkb_state,
}

// unsafe impl Send for Connection {}

impl Connection {
    #[allow(dead_code)]
    pub fn new() -> Result<Self, ConnectionError> {
        let display = unsafe { XOpenDisplay(std::ptr::null()) };
        if display.is_null() {
            return Err(ConnectionError::UnableToOpenXDisplay);
        }
        let conn = unsafe { XGetXCBConnection(display) };
        if conn.is_null() || unsafe { xcb_connection_has_error(conn) } == 1 {
            return Err(ConnectionError::UnableToGetXCBConnection);
        }
        let setup = unsafe { xcb_get_setup(conn) };
        let iter = unsafe { xcb_setup_roots_iterator(setup) };
        let screen = iter.data;

        let ewmh_layout = std::alloc::Layout::new::<xcb_ewmh_connection_t>();
        let ewmh = unsafe { std::alloc::alloc_zeroed(ewmh_layout) } as *mut xcb_ewmh_connection_t;
        let ewmh_cookies = unsafe { xcb_ewmh_init_atoms(conn, ewmh) };
        if unsafe { xcb_ewmh_init_atoms_replies(ewmh, ewmh_cookies, std::ptr::null_mut()) } != 1 {
            return Err(ConnectionError::UnableToInitEwmh);
        }
        let mut net_atoms = unsafe {
            [
                (*ewmh)._NET_SUPPORTED,
                (*ewmh)._NET_SUPPORTING_WM_CHECK,
                (*ewmh)._NET_DESKTOP_NAMES,
                (*ewmh)._NET_DESKTOP_VIEWPORT,
                (*ewmh)._NET_NUMBER_OF_DESKTOPS,
                (*ewmh)._NET_CURRENT_DESKTOP,
                (*ewmh)._NET_CLIENT_LIST,
                (*ewmh)._NET_ACTIVE_WINDOW,
                (*ewmh)._NET_WM_NAME,
                (*ewmh)._NET_CLOSE_WINDOW,
                (*ewmh)._NET_WM_STRUT_PARTIAL,
                (*ewmh)._NET_WM_DESKTOP,
                (*ewmh)._NET_WM_STATE,
                (*ewmh)._NET_WM_STATE_HIDDEN,
                (*ewmh)._NET_WM_STATE_FULLSCREEN,
                (*ewmh)._NET_WM_STATE_BELOW,
                (*ewmh)._NET_WM_STATE_ABOVE,
                (*ewmh)._NET_WM_STATE_STICKY,
                (*ewmh)._NET_WM_STATE_DEMANDS_ATTENTION,
                (*ewmh)._NET_WM_WINDOW_TYPE,
                (*ewmh)._NET_WM_WINDOW_TYPE_DOCK,
                (*ewmh)._NET_WM_WINDOW_TYPE_DESKTOP,
                (*ewmh)._NET_WM_WINDOW_TYPE_NOTIFICATION,
                (*ewmh)._NET_WM_WINDOW_TYPE_DIALOG,
                (*ewmh)._NET_WM_WINDOW_TYPE_SPLASH,
                (*ewmh)._NET_WM_WINDOW_TYPE_UTILITY,
                (*ewmh)._NET_WM_WINDOW_TYPE_TOOLBAR,
            ]
        };
        let set_supp_cookie = unsafe {
            xcb_ewmh_set_supported_checked(ewmh, 0, net_atoms.len() as u32, net_atoms.as_mut_ptr())
        };
        let set_supp_error = unsafe { xcb_request_check(conn, set_supp_cookie) };
        if !set_supp_error.is_null() {
            return Err(ConnectionError::UnableToSetSupportedEwmhAtoms);
        }

        let randr_ext_name = "RANDR";
        let randr_reply = unsafe {
            xcb_query_extension_reply(
                conn,
                xcb_query_extension(
                    conn,
                    randr_ext_name.len() as u16,
                    randr_ext_name.as_ptr() as *const i8,
                ),
                std::ptr::null_mut(),
            )
        };
        if randr_reply.is_null() {
            return Err(ConnectionError::UnableToQueryRandrExtension(
                "returned null".to_owned(),
            ));
        }
        if unsafe { *randr_reply }.present == 0 {
            return Err(ConnectionError::UnableToQueryRandrExtension(
                "returned 'present == 0'".to_owned(),
            ));
        }
        let randr_event_base = unsafe { *randr_reply }.first_event;

        let randr_select_cookie = unsafe {
            xcb_randr_select_input_checked(
                conn,
                (*screen).root,
                XCB_RANDR_NOTIFY_MASK_SCREEN_CHANGE as u16,
            )
        };
        let randr_select_error = unsafe { xcb_request_check(conn, randr_select_cookie) };
        if !randr_select_error.is_null() {
            return Err(ConnectionError::UnableToQueryRandrExtension(format!(
                "{:?}",
                randr_select_error
            )));
        }

        let mut xkb_event_base: u8 = 0;
        unsafe {
            xkb_x11_setup_xkb_extension(
                conn,
                XKB_X11_MIN_MAJOR_XKB_VERSION as u16,
                XKB_X11_MIN_MINOR_XKB_VERSION as u16,
                XKB_X11_SETUP_XKB_EXTENSION_NO_FLAGS,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut xkb_event_base,
                std::ptr::null_mut(),
            );
        }
        let xkb_device_id = unsafe { xkb_x11_get_core_keyboard_device_id(conn) };
        let xkb_context = unsafe { xkb_context_new(XKB_CONTEXT_NO_FLAGS) };
        let xkb_keymap = unsafe {
            xkb_x11_keymap_new_from_device(
                xkb_context,
                conn,
                xkb_device_id,
                XKB_KEYMAP_COMPILE_NO_FLAGS,
            )
        };
        let xkb_state = unsafe { xkb_x11_state_new_from_device(xkb_keymap, conn, xkb_device_id) };
        let mask = XCB_XKB_EVENT_TYPE_STATE_NOTIFY
            | XCB_XKB_EVENT_TYPE_CONTROLS_NOTIFY
            | XCB_XKB_EVENT_TYPE_NAMES_NOTIFY;
        unsafe {
            xcb_xkb_select_events(
                conn,
                xkb_device_id as u16,
                mask as u16,
                0,
                mask as u16,
                0,
                0,
                std::ptr::null_mut(),
            )
        };

        Ok(Self {
            display,
            conn,
            screen,
            ewmh,
            xkb_event_base,
            xkb_device_id,
            xkb_context,
            xkb_keymap,
            xkb_state,
            randr_event_base,
        })
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe { xcb_disconnect(self.conn) };
        unsafe { XCloseDisplay(self.display) };
    }
}

impl Connection {
    #[inline]
    pub fn root(&self) -> xcb_window_t {
        unsafe { *self.screen }.root
    }

    #[inline]
    pub fn screen(&self) -> xcb_screen_t {
        unsafe { *self.screen }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn screen_rect(&self) -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: unsafe { *self.screen }.width_in_pixels as u32,
            height: unsafe { *self.screen }.height_in_pixels as u32,
        }
    }

    #[allow(dead_code)]
    pub fn change_window_attrs_checked(
        &self,
        window: xcb_window_t,
        mask: xcb_cw_t,
        values: xcb_event_mask_t,
    ) -> Result<(), ConnectionError> {
        let values = [values];
        let cookie = unsafe {
            xcb_change_window_attributes_checked(
                self.conn,
                window,
                mask,
                values.as_ptr() as *const ::std::os::raw::c_void,
            )
        };
        let generic_error = unsafe { xcb_request_check(self.conn, cookie) };
        if !generic_error.is_null() {
            return Err(ConnectionError::UnableToChangeWindowAttrs((
                window,
                format!(
                    "error_code: {}, major_code: {}, minor_code: {}",
                    unsafe { *generic_error }.error_code,
                    unsafe { *generic_error }.major_code,
                    unsafe { *generic_error }.minor_code
                ),
            )));
        }

        Ok(())
    }

    #[allow(dead_code)]
    #[inline]
    pub fn change_window_attrs(
        &self,
        window: xcb_window_t,
        mask: xcb_cw_t,
        values: xcb_event_mask_t,
    ) {
        let values = [values];
        unsafe {
            xcb_change_window_attributes(
                self.conn,
                window,
                mask,
                values.as_ptr() as *const ::std::os::raw::c_void,
            )
        };
    }

    pub fn grab_pointer(
        &self,
        mask_pass_through: xcb_event_mask_t,
        window: xcb_window_t,
        window_stay_in: xcb_window_t,
    ) -> Result<(), ConnectionError> {
        let cookie = unsafe {
            xcb_grab_pointer(
                self.conn,
                0 as u8, /* get all pointer events specified by the following mask */
                window,
                mask_pass_through as u16, /* which events to let through */
                XCB_GRAB_MODE_ASYNC as u8,
                XCB_GRAB_MODE_ASYNC as u8,
                window_stay_in, /* confine_to = in which window should the cursor stay */
                XCB_NONE,
                XCB_CURRENT_TIME,
            )
        };
        let mut error: *mut xcb_generic_error_t = std::ptr::null_mut();
        let reply = unsafe { xcb_grab_pointer_reply(self.conn, cookie, &mut error) };
        _ = reply;
        if !error.is_null() {
            Err(ConnectionError::UnableToGrabPointer((
                window_stay_in,
                format!("{:?}", error),
            )))
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn ungrab_pointer(&self) {
        unsafe { xcb_ungrab_pointer(self.conn, XCB_CURRENT_TIME) };
    }

    #[allow(dead_code)]
    pub fn allow_events(&self, mask: xcb_allow_t) {
        unsafe { xcb_allow_events(self.conn, mask as u8, XCB_CURRENT_TIME) };
    }

    #[allow(dead_code)]
    pub fn change_cursor(&self, new_name: &str) {
        if let Ok(cstr) = std::ffi::CString::new(new_name) {
            let mut cursor_ctx: *mut xcb_cursor_context_t = std::ptr::null_mut();
            if unsafe { xcb_cursor_context_new(self.conn, self.screen, &mut cursor_ctx) } == 0 {
                let cursor = unsafe {
                    xcb_cursor_load_cursor(
                        cursor_ctx,
                        cstr.as_ptr() as *const ::std::os::raw::c_char,
                    )
                };
                self.change_window_attrs(self.root(), XCB_CW_CURSOR, cursor);
                unsafe {
                    xcb_cursor_context_free(cursor_ctx);
                }
            }
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn flush(&self) {
        unsafe { xcb_flush(self.conn) };
    }

    #[allow(dead_code)]
    pub fn has_override_redirect(&self, window: xcb_window_t) -> bool {
        let attrs_reply = unsafe {
            xcb_get_window_attributes_reply(
                self.conn,
                xcb_get_window_attributes(self.conn, window),
                std::ptr::null_mut(),
            )
        };
        return !attrs_reply.is_null() && unsafe { *attrs_reply }.override_redirect == 1;
    }

    #[allow(dead_code)]
    pub fn window_requested_workspace(&self, window: xcb_window_t) -> Option<u32> {
        let mut workspace: MaybeUninit<u32> = MaybeUninit::uninit();
        let mut error: *mut xcb_generic_error_t = std::ptr::null_mut();
        let res = unsafe {
            xcb_ewmh_get_cardinal_reply(
                self.ewmh,
                xcb_ewmh_get_wm_desktop(self.ewmh, window),
                workspace.as_mut_ptr(),
                &mut error,
            )
        };
        if res == 1 && error.is_null() {
            let index = unsafe { workspace.assume_init() };
            if index < 10 {
                return Some(index);
            }
        }
        None
    }

    /// Returns Class name + Instance name
    #[allow(dead_code)]
    pub fn window_class_instance_names(&self, window: xcb_window_t) -> Option<(String, String)> {
        let mut error: *mut xcb_generic_error_t = std::ptr::null_mut();
        let reply = unsafe {
            xcb_get_property_reply(
                self.conn,
                xcb_get_property(
                    self.conn,
                    0,
                    window,
                    XCB_ATOM_WM_CLASS,
                    XCB_GET_PROPERTY_TYPE_ANY,
                    0,
                    1024,
                ),
                &mut error,
            )
        };
        if !reply.is_null() && error.is_null() {
            let value_ptr = unsafe { xcb_get_property_value(reply) as *const u8 };
            let value_len = unsafe { xcb_get_property_value_length(reply) as usize };
            let data = unsafe { std::slice::from_raw_parts(value_ptr, value_len) };
            let mut parts = data.split(|&b| b == 0);
            let class_name = parts
                .next()
                .map(|s| String::from_utf8_lossy(s).into_owned());
            let instance_name = parts
                .next()
                .map(|s| String::from_utf8_lossy(s).into_owned());
            if class_name.is_some() && instance_name.is_some() {
                return Some((class_name.unwrap(), instance_name.unwrap()));
            }
        }
        None
    }

    #[allow(dead_code)]
    pub fn window_rect_hints(&self, window: xcb_window_t) -> Option<Rect> {
        let mut error: *mut xcb_generic_error_t = std::ptr::null_mut();
        let reply = unsafe {
            xcb_get_property_reply(
                self.conn,
                xcb_get_property(
                    self.conn,
                    0,
                    window,
                    XCB_ATOM_WM_NORMAL_HINTS,
                    XCB_GET_PROPERTY_TYPE_ANY,
                    0,
                    1024,
                ),
                &mut error,
            )
        };
        if !reply.is_null() && error.is_null() {
            unsafe {
                let size_hints = xcb_get_property_value(reply) as *mut xcb_size_hints_t;
                if !size_hints.is_null() {
                    return Some(Rect {
                        x: (*size_hints).x,
                        y: (*size_hints).y,
                        width: (*size_hints).width as u32,
                        height: (*size_hints).height as u32,
                    });
                }
            }
        }
        None
    }

    #[allow(dead_code)]
    pub fn window_strut_partial(&self, window: xcb_window_t, monitor_rect: &Rect) -> Option<Rect> {
        let mut error: *mut xcb_generic_error_t = std::ptr::null_mut();
        let mut maybe_strut: MaybeUninit<xcb_ewmh_wm_strut_partial_t> = MaybeUninit::uninit();
        unsafe {
            xcb_ewmh_get_wm_strut_partial_reply(
                self.ewmh,
                xcb_ewmh_get_wm_strut_partial(self.ewmh, window),
                maybe_strut.as_mut_ptr(),
                &mut error,
            )
        };
        if error.is_null() {
            let strut = unsafe { maybe_strut.assume_init() };
            if strut.left > 0 {
                if strut.left_end_y as i32 - strut.left_start_y as i32 > 1 {
                    return Some(Rect {
                        x: 0,
                        y: strut.left_start_y as i32,
                        width: strut.left,
                        height: strut.left_end_y - strut.left_start_y,
                    });
                } else {
                    return Some(Rect {
                        x: 0,
                        y: strut.left_start_y as i32,
                        width: strut.left,
                        height: monitor_rect.height,
                    });
                }
            }
            if strut.right > 0 {
                if strut.right_end_y as i32 - strut.right_start_y as i32 > 1 {
                    return Some(Rect {
                        x: monitor_rect.width as i32 - strut.right as i32,
                        y: strut.left_start_y as i32,
                        width: strut.right,
                        height: strut.right_end_y - strut.right_start_y,
                    });
                } else {
                    return Some(Rect {
                        x: monitor_rect.width as i32 - strut.right as i32,
                        y: strut.right_start_y as i32,
                        width: strut.right,
                        height: monitor_rect.height,
                    });
                }
            }
            if strut.top > 0 {
                if strut.top_end_x as i32 - strut.top_start_x as i32 > 1 {
                    return Some(Rect {
                        x: strut.top_start_x as i32,
                        y: 0,
                        width: strut.top_end_x - strut.top_start_x,
                        height: strut.top,
                    });
                } else {
                    return Some(Rect {
                        x: strut.top_start_x as i32,
                        y: 0,
                        width: monitor_rect.width,
                        height: strut.top,
                    });
                }
            }
            if strut.bottom > 0 && strut.bottom_end_x as i32 - strut.bottom_start_x as i32 > 1 {
                return Some(Rect {
                    x: strut.bottom_start_x as i32,
                    y: monitor_rect.height as i32 - strut.bottom as i32,
                    width: strut.bottom_end_x - strut.bottom_start_x,
                    height: strut.bottom,
                });
            } else {
                return Some(Rect {
                    x: 0,
                    y: monitor_rect.height as i32 - 25,
                    width: monitor_rect.width,
                    height: 25,
                });
            }
        }
        None
    }

    #[allow(dead_code)]
    #[inline]
    pub fn generate_id(&self) -> u32 {
        unsafe { xcb_generate_id(self.conn) }
    }

    #[allow(dead_code)]
    pub fn create_window<const N: usize>(
        &self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        border_width: u32,
        mask: xcb_cw_t,
        values: [u32; N],
    ) -> xcb_window_t {
        let window = self.generate_id() as xcb_window_t;
        unsafe {
            xcb_create_window(
                self.conn,
                XCB_COPY_FROM_PARENT as u8,
                window,
                self.root(),
                x as i16,
                y as i16,
                width as u16,
                height as u16,
                border_width as u16,
                XCB_WINDOW_CLASS_INPUT_OUTPUT as u16,
                (*self.screen).root_visual,
                mask,
                values.as_ptr() as *const ::std::os::raw::c_void,
            )
        };
        window
    }

    #[allow(dead_code)]
    #[inline]
    pub fn change_property(
        &self,
        window: xcb_window_t,
        property: xcb_atom_t,
        type_: xcb_atom_t,
        format: u8,
        data_len: usize,
        data: *const ::std::os::raw::c_void,
    ) {
        unsafe {
            xcb_change_property(
                self.conn,
                XCB_PROP_MODE_REPLACE as u8,
                window,
                property,
                type_,
                format,
                data_len as u32,
                data,
            )
        };
    }

    #[allow(dead_code)]
    #[inline]
    pub fn window_set_atom(&self, window: xcb_window_t, name: &xcb_atom_t, value: &xcb_atom_t) {
        self.change_property(
            window,
            *name,
            XCB_ATOM_ATOM,
            32,
            1,
            value as *const xcb_atom_t as *const ::std::os::raw::c_void,
        );
    }

    #[allow(dead_code)]
    pub fn window_set_instance_class_names(
        &self,
        window: xcb_window_t,
        instance_name: &str,
        class_name: &str,
    ) {
        let mut wm_class_property = String::from(instance_name);
        wm_class_property.push('\0');
        wm_class_property.push_str(class_name);
        wm_class_property.push('\0');
        self.change_property(
            window,
            XCB_ATOM_WM_CLASS,
            XCB_ATOM_STRING,
            8,
            wm_class_property.len(),
            wm_class_property.as_ptr() as *const ::std::os::raw::c_void,
        );
    }

    #[allow(dead_code)]
    pub fn window_set_wm_normal_hints_size(&self, window: xcb_window_t, width: u32, height: u32) {
        let mut hints = xcb_size_hints_t {
            flags: 0,
            x: 0,
            y: 0,
            width: width as i32,
            height: height as i32,
            min_width: 0,
            min_height: 0,
            max_width: 0,
            max_height: 0,
            width_inc: 0,
            height_inc: 0,
            min_aspect_num: 0,
            min_aspect_den: 0,
            max_aspect_num: 0,
            max_aspect_den: 0,
            base_width: 0,
            base_height: 0,
            win_gravity: 0,
        };
        unsafe { xcb_icccm_set_wm_normal_hints(self.conn, window, &mut hints) };
    }

    #[allow(dead_code)]
    pub fn intern_atom(&self, name: &str) -> Option<xcb_atom_t> {
        let mut error: *mut xcb_generic_error_t = std::ptr::null_mut();
        let reply = unsafe {
            xcb_intern_atom_reply(
                self.conn,
                xcb_intern_atom(
                    self.conn,
                    0,
                    name.len() as u16,
                    name.as_ptr() as *const ::std::os::raw::c_char,
                ),
                &mut error,
            )
        };
        if error.is_null() && !reply.is_null() {
            return Some(unsafe { *reply }.atom);
        }
        None
    }

    #[allow(dead_code)]
    pub fn intern_atoms<const N: usize>(&self, names: [&str; N]) -> HashMap<String, xcb_atom_t> {
        let mut atoms: HashMap<String, xcb_atom_t> = HashMap::new();

        let mut cookies: Vec<xcb_intern_atom_cookie_t> = Vec::new();
        cookies.reserve(names.len());
        for name in &names {
            let cookie = unsafe {
                xcb_intern_atom(
                    self.conn,
                    0,
                    name.len() as u16,
                    name.as_ptr() as *const ::std::os::raw::c_char,
                )
            };
            cookies.push(cookie);
        }

        let mut error: *mut xcb_generic_error_t = std::ptr::null_mut();
        for (idx, cookie) in cookies.iter().enumerate() {
            let reply = unsafe { xcb_intern_atom_reply(self.conn, *cookie, &mut error) };
            if error.is_null() && !reply.is_null() {
                let atom = unsafe { *reply }.atom;
                atoms.insert(names[idx].to_string(), atom);
            } else {
                error = std::ptr::null_mut();
            }
        }

        atoms
    }

    #[inline]
    pub fn map_window(&self, window: xcb_window_t) {
        unsafe { xcb_map_window(self.conn, window) };
    }

    #[inline]
    pub fn unmap_window(&self, window: xcb_window_t) {
        unsafe { xcb_unmap_window(self.conn, window) };
    }

    #[inline]
    pub fn window_configure(&self, window: xcb_window_t, rect: &Rect, border_width: u32) {
        let values = [
            rect.x,
            rect.y,
            rect.width as i32,
            rect.height as i32,
            border_width as i32,
        ];
        unsafe {
            xcb_configure_window(
                self.conn,
                window,
                (XCB_CONFIG_WINDOW_X
                    | XCB_CONFIG_WINDOW_Y
                    | XCB_CONFIG_WINDOW_WIDTH
                    | XCB_CONFIG_WINDOW_HEIGHT
                    | XCB_CONFIG_WINDOW_BORDER_WIDTH) as u16,
                values.as_ptr() as *const ::std::os::raw::c_void,
            )
        };
    }

    #[inline]
    pub fn window_raise(&self, window: xcb_window_t) {
        let values = [XCB_STACK_MODE_ABOVE];
        unsafe {
            xcb_configure_window(
                self.conn,
                window,
                XCB_CONFIG_WINDOW_STACK_MODE as u16,
                values.as_ptr() as *const ::std::os::raw::c_void,
            )
        };
    }

    pub fn window_configure_checked(
        &self,
        window: xcb_window_t,
        rect: &Rect,
        border_width: u32,
    ) -> Result<(), XcbErrors> {
        let values = [
            rect.x,
            rect.y,
            rect.width as i32,
            rect.height as i32,
            border_width as i32,
        ];
        let cookie = unsafe {
            xcb_configure_window_checked(
                self.conn,
                window,
                (XCB_CONFIG_WINDOW_X
                    | XCB_CONFIG_WINDOW_Y
                    | XCB_CONFIG_WINDOW_WIDTH
                    | XCB_CONFIG_WINDOW_HEIGHT
                    | XCB_CONFIG_WINDOW_BORDER_WIDTH) as u16,
                values.as_ptr() as *const ::std::os::raw::c_void,
            )
        };
        let error = unsafe { xcb_request_check(self.conn, cookie) };
        if !error.is_null() {
            return Err(XcbErrors::from(error));
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn window_set_input_focus(&self, window: xcb_window_t) {
        unsafe {
            xcb_set_input_focus(
                self.conn,
                XCB_INPUT_FOCUS_POINTER_ROOT as u8,
                window,
                XCB_CURRENT_TIME,
            )
        };
    }

    pub fn grab_key(&self, modifiers: xcb_mod_mask_t, keycode: xcb_keycode_t) {
        unsafe {
            xcb_grab_key(
                self.conn,
                1,
                self.root(),
                modifiers as u16,
                keycode as u8,
                XCB_GRAB_MODE_ASYNC as u8,
                XCB_GRAB_MODE_ASYNC as u8,
            )
        };
    }

    pub fn ungrab_key(&self, modifiers: xcb_mod_mask_t, keycode: xcb_keycode_t) {
        unsafe { xcb_ungrab_key(self.conn, keycode as u8, self.root(), modifiers as u16) };
    }

    #[allow(dead_code)]
    pub fn create_pixmap(&self, pixmap: xcb_pixmap_t, width: u32, height: u32, depth: u8) {
        unsafe {
            xcb_create_pixmap(
                self.conn,
                depth,
                pixmap,
                self.screen().root,
                width as u16,
                height as u16,
            )
        };
    }

    #[allow(dead_code)]
    pub fn create_pixmap_checked(
        &self,
        pixmap: xcb_pixmap_t,
        width: u32,
        height: u32,
        depth: u8,
    ) -> Result<(), XcbErrors> {
        let cookie = unsafe {
            xcb_create_pixmap_checked(
                self.conn,
                depth,
                pixmap,
                self.screen().root,
                width as u16,
                height as u16,
            )
        };
        let error = unsafe { xcb_request_check(self.conn, cookie) };
        if !error.is_null() {
            return Err(XcbErrors::from(error));
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn create_gc<const N: usize>(
        &self,
        gc: xcb_gcontext_t,
        pixmap: xcb_pixmap_t,
        mask: xcb_gc_t,
        values: [u32; N],
    ) {
        let value_list = if values.is_empty() {
            std::ptr::null_mut()
        } else {
            values.as_ptr() as *const ::std::os::raw::c_void
        };
        unsafe { xcb_create_gc(self.conn, gc, pixmap, mask, value_list) };
    }

    #[allow(dead_code)]
    pub fn create_gc_checked<const N: usize>(
        &self,
        gc: xcb_gcontext_t,
        pixmap: xcb_pixmap_t,
        mask: xcb_gc_t,
        values: [u32; N],
    ) -> Result<(), XcbErrors> {
        let cookie = unsafe {
            xcb_create_gc_checked(
                self.conn,
                gc,
                pixmap,
                mask,
                values.as_ptr() as *const ::std::os::raw::c_void,
            )
        };
        let error = unsafe { xcb_request_check(self.conn, cookie) };
        if !error.is_null() {
            return Err(XcbErrors::from(error));
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn change_gc<const N: usize>(&self, gc: xcb_gcontext_t, mask: xcb_gc_t, values: [u32; N]) {
        unsafe {
            xcb_change_gc(
                self.conn,
                gc,
                mask,
                values.as_ptr() as *const ::std::os::raw::c_void,
            )
        };
    }

    #[allow(dead_code)]
    pub fn poly_point(&self, pixmap: xcb_pixmap_t, gc: xcb_gcontext_t, x: i16, y: i16) {
        let points = [xcb_point_t { x, y }];
        unsafe {
            xcb_poly_point(
                self.conn,
                XCB_COORD_MODE_ORIGIN as u8,
                pixmap,
                gc,
                points.len() as u32,
                points.as_ptr(),
            )
        };
    }

    #[allow(dead_code)]
    pub fn poly_fill_rectangle(
        &self,
        pixmap: xcb_pixmap_t,
        gc: xcb_gcontext_t,
        rect: xcb_rectangle_t,
    ) {
        let rects = [rect];
        unsafe {
            xcb_poly_fill_rectangle(self.conn, pixmap, gc, rects.len() as u32, rects.as_ptr())
        };
    }

    #[allow(dead_code)]
    pub fn poly_fill_arc<const N: usize>(
        &self,
        pixmap: xcb_pixmap_t,
        gc: xcb_gcontext_t,
        arcs: [xcb_arc_t; N],
    ) {
        unsafe { xcb_poly_fill_arc(self.conn, pixmap, gc, arcs.len() as u32, arcs.as_ptr()) };
    }

    #[allow(dead_code)]
    pub fn put_image_rgba(
        &self,
        pixmap: xcb_pixmap_t,
        gc: xcb_gcontext_t,
        width: u32,
        height: u32,
        data_len: usize,
        data: *const u8,
    ) {
        unsafe {
            xcb_put_image(
                self.conn,
                XCB_IMAGE_FORMAT_Z_PIXMAP as u8,
                pixmap,
                gc,
                width as u16,
                height as u16,
                0,
                0,
                0,
                4, // channels
                data_len as u32,
                data,
            )
        };
    }

    #[allow(dead_code)]
    pub fn image_put_checked(
        &self,
        pixmap: xcb_pixmap_t,
        gc: xcb_gcontext_t,
        width: u32,
        height: u32,
        data: *mut u8,
        data_len: usize,
    ) -> Result<(), XcbErrors> {
        let image = unsafe {
            xcb_image_create_native(
                self.conn,
                width as u16,
                height as u16,
                XCB_IMAGE_FORMAT_Z_PIXMAP,
                self.screen().root_depth,
                data as *mut ::std::os::raw::c_void,
                data_len as u32,
                data,
            )
        };
        if image.is_null() {
            return Err(XcbErrors::FailedToCreateImage);
        }
        let cookie = unsafe { xcb_image_put(self.conn, pixmap, gc, image, 0, 0, 0) };
        let error = unsafe { xcb_request_check(self.conn, cookie) };
        unsafe { xcb_image_destroy(image) };
        if !error.is_null() {
            return Err(XcbErrors::from(error));
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn image_create_checked(
        &self,
        pixmap: xcb_pixmap_t,
        gc: xcb_gcontext_t,
        width: u32,
        height: u32,
        data: *mut u8,
        data_len: usize,
    ) -> Result<(), XcbErrors> {
        let image = unsafe {
            xcb_image_create(
                width as u16,
                height as u16,
                XCB_IMAGE_FORMAT_Z_PIXMAP,
                32,                       // xpad
                self.screen().root_depth, // depth
                24,                       // bpp
                32,                       // unit
                XCB_IMAGE_ORDER_LSB_FIRST,
                XCB_IMAGE_ORDER_LSB_FIRST,
                std::ptr::null_mut(), // base
                data_len as u32,      // bytes
                data,
            )
        };
        if image.is_null() {
            return Err(XcbErrors::FailedToCreateImage);
        }
        let cookie = unsafe { xcb_image_put(self.conn, pixmap, gc, image, 0, 0, 0) };
        let error = unsafe { xcb_request_check(self.conn, cookie) };
        unsafe { xcb_image_destroy(image) };
        if !error.is_null() {
            return Err(XcbErrors::from(error));
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn image_create_native_checked(
        &self,
        pixmap: xcb_pixmap_t,
        gc: xcb_gcontext_t,
        width: u32,
        height: u32,
        data: *mut u8,
        data_len: usize,
    ) -> Result<(), XcbErrors> {
        let image = unsafe {
            xcb_image_create_native(
                self.conn,
                width as u16,
                height as u16,
                XCB_IMAGE_FORMAT_XY_PIXMAP,
                self.screen().root_depth,
                std::ptr::null_mut(), //data as *mut std::os::raw::c_void,
                data_len as u32,
                data,
            )
        };
        if image.is_null() {
            return Err(XcbErrors::FailedToCreateNativeImage);
        }
        let cookie = unsafe { xcb_image_put(self.conn, pixmap, gc, image, 0, 0, 0) };
        unsafe { xcb_image_destroy(image) };
        let error = unsafe { xcb_request_check(self.conn, cookie) };
        if !error.is_null() {
            return Err(XcbErrors::from(error));
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn create_cursor(
        &self,
        cursor: xcb_cursor_t,
        pixmap: xcb_pixmap_t,
        mask_pixmap: xcb_pixmap_t,
    ) {
        unsafe {
            xcb_create_cursor(
                self.conn,
                cursor,
                pixmap,
                mask_pixmap,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            )
        };
    }

    #[allow(dead_code)]
    pub fn create_cursor_checked(
        &self,
        cursor: xcb_cursor_t,
        pixmap: xcb_pixmap_t,
        mask_pixmap: xcb_pixmap_t,
    ) -> Result<(), XcbErrors> {
        let cookie = unsafe {
            xcb_create_cursor_checked(
                self.conn,
                cursor,
                pixmap,
                mask_pixmap,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            )
        };
        let error = unsafe { xcb_request_check(self.conn, cookie) };
        if !error.is_null() {
            return Err(XcbErrors::from(error));
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn free_gc(&self, gc: xcb_gcontext_t) {
        unsafe { xcb_free_gc(self.conn, gc) };
    }

    #[allow(dead_code)]
    pub fn free_pixmap(&self, pixmap: xcb_pixmap_t) {
        unsafe { xcb_free_pixmap(self.conn, pixmap) };
    }

    #[allow(dead_code)]
    pub fn set_cursor_filename(&self, filename: &str) -> Result<(), XcbErrors> {
        if let Ok(filename) = CString::new(filename) {
            let cursor = unsafe { XcursorFilenameLoadCursor(self.display, filename.as_ptr()) };
            if cursor == 0 {
                return Err(XcbErrors::FailedToCreateCursor(
                    "XcursorFilenameLoadCursor return None".to_string(),
                ));
            }
            unsafe { XDefineCursor(self.display, XDefaultRootWindow(self.display), cursor) };
        } else {
            return Err(XcbErrors::FailedToCreateCursor(format!(
                "failed to convert filename '{}' to cstring",
                filename
            )));
        }
        Ok(())
    }

    #[allow(dead_code)]
    #[inline]
    pub fn shape_mask(&self, window: xcb_window_t, mask: xcb_pixmap_t) {
        unsafe {
            xcb_shape_mask(
                self.conn,
                XCB_SHAPE_SO_SET as u8,
                XCB_SHAPE_SK_BOUNDING as u8,
                window,
                0,
                0,
                mask,
            )
        };
    }

    #[allow(dead_code)]
    pub fn window_destroy_gracefully(&self, window: xcb_window_t) {
        const WM_PROTOCOLS_ATOM_NAME: &str = "WM_PROTOCOLS";
        const WM_DELETE_WINDOW_ATOM_NAME: &str = "WM_DELETE_WINDOW";
        let atoms = self.intern_atoms([WM_PROTOCOLS_ATOM_NAME, WM_DELETE_WINDOW_ATOM_NAME]);
        if atoms.contains_key(WM_PROTOCOLS_ATOM_NAME)
            && atoms.contains_key(WM_DELETE_WINDOW_ATOM_NAME)
        {
            let protocols_atom = *atoms.get(WM_PROTOCOLS_ATOM_NAME).unwrap();
            let delete_atom = *atoms.get(WM_DELETE_WINDOW_ATOM_NAME).unwrap();

            let mut error: *mut xcb_generic_error_t = std::ptr::null_mut();
            let reply = unsafe {
                xcb_get_property_reply(
                    self.conn,
                    xcb_get_property(self.conn, 0, window, protocols_atom, XCB_ATOM_ATOM, 0, 32),
                    &mut error,
                )
            };
            if !reply.is_null() && error.is_null() {
                let atoms = unsafe { xcb_get_property_value(reply) as *mut xcb_atom_t };
                let atoms_count = unsafe { xcb_get_property_value_length(reply) as usize };
                if !atoms.is_null() {
                    let mut found = false;
                    for index in 0..atoms_count {
                        unsafe {
                            let atom = atoms.add(index);
                            if *atom == delete_atom {
                                found = true;
                                break;
                            }
                        }
                    }
                    if found {
                        let message_data = xcb_client_message_data_t {
                            data32: [delete_atom, XCB_CURRENT_TIME, 0, 0, 0],
                        };
                        let event = xcb_client_message_event_t {
                            response_type: XCB_CLIENT_MESSAGE as u8,
                            format: 32,
                            sequence: 0,
                            window,
                            type_: protocols_atom,
                            data: message_data,
                        };
                        unsafe {
                            xcb_send_event(
                                self.conn,
                                0,
                                window,
                                XCB_EVENT_MASK_NO_EVENT,
                                &event as *const xcb_client_message_event_t as *const i8,
                            )
                        };
                        return;
                    }
                }
            }
        }
        unsafe {
            xcb_destroy_window(self.conn, window);
        }
    }

    #[allow(dead_code)]
    pub fn window_exists(&self, window: xcb_window_t) -> bool {
        let attrs_reply = unsafe {
            xcb_get_window_attributes_reply(
                self.conn,
                xcb_get_window_attributes(self.conn, window),
                std::ptr::null_mut(),
            )
        };
        !attrs_reply.is_null()
    }

    #[allow(dead_code)]
    pub fn window_destroy(&self, window: xcb_window_t) {
        unsafe { xcb_destroy_window(self.conn, window) };
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WindowType {
    Normal,
    Floating,
    Docked,
}

impl Connection {
    #[allow(dead_code)]
    pub fn window_type(&self, window: xcb_window_t) -> WindowType {
        let mut atoms_reply: MaybeUninit<xcb_ewmh_get_atoms_reply_t> = MaybeUninit::uninit();
        let res = unsafe {
            xcb_ewmh_get_wm_window_type_reply(
                self.ewmh,
                xcb_ewmh_get_wm_window_type(self.ewmh, window),
                atoms_reply.as_mut_ptr(),
                std::ptr::null_mut(),
            )
        };
        let mut window_type = WindowType::Normal;
        if res == 1 {
            unsafe {
                let window_type_atoms = atoms_reply.assume_init();
                let length = window_type_atoms.atoms_len as usize;
                for i in 0..length {
                    let atom = *(window_type_atoms.atoms.add(i));
                    if atom == (*self.ewmh)._NET_WM_WINDOW_TYPE_NORMAL {
                        break;
                    }
                    if atom == (*self.ewmh)._NET_WM_WINDOW_TYPE_DOCK {
                        window_type = WindowType::Docked;
                        break;
                    }
                    if atom == (*self.ewmh)._NET_WM_WINDOW_TYPE_DIALOG
                        || atom == (*self.ewmh)._NET_WM_WINDOW_TYPE_UTILITY
                        || atom == (*self.ewmh)._NET_WM_WINDOW_TYPE_TOOLBAR
                        || atom == (*self.ewmh)._NET_WM_WINDOW_TYPE_SPLASH
                        || atom == (*self.ewmh)._NET_WM_WINDOW_TYPE_MENU
                    {
                        window_type = WindowType::Floating;
                        break;
                    }
                }
                xcb_ewmh_get_atoms_reply_wipe(atoms_reply.as_mut_ptr());
            }
        }
        window_type
    }
}

#[repr(u8)]
pub enum MouseButton {
    Left,
    Right,
}

impl Connection {
    pub fn grab_button(&self, mouse_button: MouseButton) {
        unsafe {
            xcb_grab_button(
                self.conn,
                0,
                self.root(),
                XCB_EVENT_MASK_BUTTON_PRESS as u16,
                XCB_GRAB_MODE_ASYNC as u8,
                XCB_GRAB_MODE_ASYNC as u8,
                XCB_NONE,
                XCB_NONE,
                mouse_button as u8,
                XCB_MOD_MASK_ANY as u16,
            )
        };
    }
}

#[derive(Debug)]
pub struct XcbError {
    pub error_code: u8,
    pub major_code: u8,
    pub minor_code: u16,
}

impl XcbError {
    fn new(error: *mut xcb_generic_error_t) -> Self {
        Self {
            error_code: unsafe { *error }.error_code,
            major_code: unsafe { *error }.major_code,
            minor_code: unsafe { *error }.minor_code,
        }
    }
}

#[derive(Debug)]
pub enum XcbErrors {
    Window(XcbError),
    Pixmap(XcbError),
    Atom(XcbError),
    Cursor(XcbError),
    Font(XcbError),
    Match(XcbError),
    Drawable(XcbError),
    Access(XcbError),
    Alloc(XcbError),
    Colormap(XcbError),
    GContext(XcbError),
    IdChoice(XcbError),
    Name(XcbError),
    Length(XcbError),
    Implementation(XcbError),
    UnknownError(XcbError),
    FailedToCreateImage,
    FailedToCreateNativeImage,
    FailedToCreateCursor(String),
}

impl From<*mut xcb_generic_error_t> for XcbErrors {
    fn from(error: *mut xcb_generic_error_t) -> Self {
        let error_code = unsafe { *error }.error_code as u32;
        match error_code {
            XCB_WINDOW => XcbErrors::Window(XcbError::new(error)),
            XCB_PIXMAP => XcbErrors::Pixmap(XcbError::new(error)),
            XCB_ATOM => XcbErrors::Atom(XcbError::new(error)),
            XCB_CURSOR => XcbErrors::Cursor(XcbError::new(error)),
            XCB_FONT => XcbErrors::Font(XcbError::new(error)),
            XCB_MATCH => XcbErrors::Match(XcbError::new(error)),
            XCB_DRAWABLE => XcbErrors::Drawable(XcbError::new(error)),
            XCB_ACCESS => XcbErrors::Access(XcbError::new(error)),
            XCB_ALLOC => XcbErrors::Alloc(XcbError::new(error)),
            XCB_COLORMAP => XcbErrors::Colormap(XcbError::new(error)),
            XCB_G_CONTEXT => XcbErrors::GContext(XcbError::new(error)),
            XCB_ID_CHOICE => XcbErrors::IdChoice(XcbError::new(error)),
            XCB_NAME => XcbErrors::Name(XcbError::new(error)),
            XCB_LENGTH => XcbErrors::Length(XcbError::new(error)),
            XCB_IMPLEMENTATION => XcbErrors::Implementation(XcbError::new(error)),
            _ => XcbErrors::UnknownError(XcbError::new(error)),
        }
    }
}

#[derive(Debug)]
pub enum XcbEvents {
    KeyPress {
        modifier: u16,
        keycode: xcb_keycode_t,
    },
    MapRequst {
        window: xcb_window_t,
    },
    FocusIn {
        window: xcb_window_t,
        mode: xcb_notify_mode_t,
    },
    FocusOut {
        window: xcb_window_t,
        mode: xcb_notify_mode_t,
    },
    MotionNotify {
        x: i32,
        y: i32,
        window: xcb_window_t,
        state: u32,
    },
    EnterNotify {
        window: xcb_window_t,
    },
    LeaveNotify {
        window: xcb_window_t,
    },
    ButtonPress {
        x: i32,
        y: i32,
        window: xcb_window_t,
        state: u16,
        detail: xcb_button_t,
    },
    ButtonRelease {
        x: i32,
        y: i32,
    },
    XkbStateNotify {
        event: *mut xcb_xkb_state_notify_event_t,
    },
    RandrScreenChange {
        width: u16,
        height: u16,
    },
    DestroyNotify {
        window: xcb_window_t,
    },
    UnmapNotify {
        window: xcb_window_t,
    }
}

impl Connection {
    pub fn get_file_descriptor(&self) -> RawFd {
        unsafe { xcb_get_file_descriptor(self.conn) }
    }

    pub fn wait_for_event(&self) -> Option<Result<XcbEvents, XcbErrors>> {
        let generic_event = unsafe { xcb_wait_for_event(self.conn) };
        self.process_event(generic_event)
    }

    pub fn poll_for_event(&self) -> Option<Result<XcbEvents, XcbErrors>> {
        let generic_event = unsafe { xcb_poll_for_event(self.conn) };
        self.process_event(generic_event)
    }

    fn process_event(
        &self,
        generic_event: *mut xcb_generic_event_t,
    ) -> Option<Result<XcbEvents, XcbErrors>> {
        if generic_event.is_null() {
            return None;
        }
        let response_type = unsafe { *generic_event }.response_type;
        if response_type == self.randr_event_base + XCB_RANDR_SCREEN_CHANGE_NOTIFY as u8 {
            let randr_event = generic_event as *mut xcb_randr_notify_event_t;
            if unsafe { *randr_event }.subCode == XCB_RANDR_NOTIFY as u8 {
                let event = randr_event as *mut xcb_randr_screen_change_notify_event_t;
                return Some(Ok(XcbEvents::RandrScreenChange {
                    width: unsafe { *event }.width,
                    height: unsafe { *event }.height,
                }));
            }
        }

        if response_type == self.xkb_event_base {
            let xkb_event = generic_event as *mut xcb_xkb_state_notify_event_t;
            let xkb_type = unsafe { *xkb_event }.xkbType as u32;
            return match xkb_type {
                XCB_XKB_STATE_NOTIFY => Some(Ok(XcbEvents::XkbStateNotify { event: xkb_event })),
                _ => None,
            };
        }

        let event_type = response_type & !0x80;
        // println!(
        //     "response_type: {}, event_type: {}, randr_event_base: {}, XCB_RANDR_NOTIFY: {}",
        //     response_type, event_type, self.randr_event_base, XCB_RANDR_NOTIFY
        // );
        match event_type as u32 {
            0 => {
                let error = generic_event as *mut xcb_generic_error_t;
                Some(Err(XcbErrors::from(error)))
            }
            XCB_KEY_PRESS => {
                let event = generic_event as *mut xcb_key_press_event_t;
                Some(Ok(XcbEvents::KeyPress {
                    modifier: unsafe { *event }.state,
                    keycode: unsafe { *event }.detail,
                }))
            }
            XCB_MAP_REQUEST => {
                let event = generic_event as *mut xcb_map_request_event_t;
                Some(Ok(XcbEvents::MapRequst {
                    window: unsafe { *event }.window,
                }))
            }
            XCB_FOCUS_IN => {
                let event = generic_event as *mut xcb_focus_in_event_t;
                Some(Ok(XcbEvents::FocusIn {
                    window: unsafe { *event }.event,
                    mode: unsafe { *event }.mode as xcb_notify_mode_t,
                }))
            }
            XCB_FOCUS_OUT => {
                let event = generic_event as *mut xcb_focus_out_event_t;
                Some(Ok(XcbEvents::FocusOut {
                    window: unsafe { *event }.event,
                    mode: unsafe { *event }.mode as xcb_notify_mode_t,
                }))
            }
            XCB_MOTION_NOTIFY => {
                let event = generic_event as *mut xcb_motion_notify_event_t;
                Some(Ok(XcbEvents::MotionNotify {
                    x: unsafe { *event }.event_x as i32,
                    y: unsafe { *event }.event_y as i32,
                    window: unsafe { *event }.event,
                    state: unsafe { *event }.state as u32,
                }))
            }
            XCB_BUTTON_PRESS => {
                let event = generic_event as *mut xcb_button_press_event_t;
                Some(Ok(XcbEvents::ButtonPress {
                    x: unsafe { *event }.event_x as i32,
                    y: unsafe { *event }.event_y as i32,
                    window: unsafe { *event }.child,
                    state: unsafe { *event }.state,
                    detail: unsafe { *event }.detail,
                }))
            }
            XCB_BUTTON_RELEASE => {
                let event = generic_event as *mut xcb_button_release_event_t;
                Some(Ok(XcbEvents::ButtonRelease {
                    x: unsafe { *event }.event_x as i32,
                    y: unsafe { *event }.event_y as i32,
                }))
            }
            XCB_ENTER_NOTIFY => {
                let event = generic_event as *mut xcb_enter_notify_event_t;
                Some(Ok(XcbEvents::EnterNotify {
                    window: unsafe { *event }.event,
                }))
            }
            XCB_LEAVE_NOTIFY => {
                let event = generic_event as *mut xcb_leave_notify_event_t;
                Some(Ok(XcbEvents::LeaveNotify {
                    window: unsafe { *event }.event,
                }))
            }
            XCB_DESTROY_NOTIFY => {
                let event = generic_event as *mut xcb_destroy_notify_event_t;
                Some(Ok(XcbEvents::DestroyNotify {
                    window: unsafe { *event }.window,
                }))
            }
            XCB_UNMAP_NOTIFY => {
                let event = generic_event as *mut xcb_unmap_notify_event_t;
                Some(Ok(XcbEvents::UnmapNotify {
                    window: unsafe { *event }.window,
                }))
            }
            _ => None,
        }
    }
}

impl Connection {
    pub fn xkb_state_update_mask(&self, state_notify_event: *mut xcb_xkb_state_notify_event_t) {
        unsafe {
            if (*state_notify_event).xkbType == XCB_XKB_STATE_NOTIFY as u8 {
                xkb_state_update_mask(
                    self.xkb_state,
                    (*state_notify_event).mods as u32,
                    (*state_notify_event).latchedMods as u32,
                    (*state_notify_event).lockedMods as u32,
                    (*state_notify_event).baseGroup as u32,
                    (*state_notify_event).latchedGroup as u32,
                    (*state_notify_event).lockedGroup as u32,
                );
            }
        };
    }

    pub fn xkb_keymap_get_as_string(&self) -> Option<Result<String, std::str::Utf8Error>> {
        let cstr_ptr =
            unsafe { xkb_keymap_get_as_string(self.xkb_keymap, XKB_KEYMAP_FORMAT_TEXT_V1) };
        if cstr_ptr.is_null() {
            None
        } else {
            let cstr = unsafe { std::ffi::CStr::from_ptr(cstr_ptr) };
            Some(cstr.to_str().map(|s| s.to_owned()))
        }
    }

    // pub fn xkb_layout_name(
    //     &self,
    //     idx: xkb_layout_index_t,
    // ) -> Option<Result<String, std::str::Utf8Error>> {
    //     let cstr_ptr = unsafe { xkb_keymap_layout_get_name(self.xkb_keymap, idx) };
    //     if cstr_ptr.is_null() {
    //         None
    //     } else {
    //         let cstr = unsafe { std::ffi::CStr::from_ptr(cstr_ptr) };
    //         Some(cstr.to_str().map(|s| s.to_owned()))
    //     }
    // }

    pub fn xkb_get_layout_names(&self) -> Vec<String> {
        let mut layout_names = Vec::new();
        unsafe {
            let keyboard = XkbAllocKeyboard();
            (*keyboard).dpy = self.display;
            XkbGetControls(self.display, XkbAllControlsMaskConst as u64, keyboard);
            XkbGetNames(self.display, XkbSymbolsNameMask, keyboard);
            XkbGetNames(self.display, XkbGroupNamesMask, keyboard);

            let num_groups = (*(*keyboard).ctrls).num_groups as usize;

            let symbols = (*(*keyboard).names).symbols;
            let symbol_names_cstr_ptr = XGetAtomName(self.display, symbols);
            let symbol_names_cstr = std::ffi::CStr::from_ptr(symbol_names_cstr_ptr);
            if let Ok(symbol_names) = symbol_names_cstr.to_str() {
                let parts = symbol_names.split('+');
                for part in parts {
                    if !part.starts_with("pc")
                        && !part.starts_with("inet")
                        && !part.starts_with("group")
                    {
                        let layout_name = if part.contains('(') {
                            part.chars().take_while(|c| *c != '(').collect()
                        } else if part.contains(':') {
                            part.chars().take_while(|c| *c != ':').collect()
                        } else {
                            part.to_owned()
                        };
                        layout_names.push(layout_name);
                    }
                }
            } else {
                let groups = (*(*keyboard).names).groups;
                for group_atom in &groups[0..num_groups] {
                    let group_name_cstr_ptr = XGetAtomName(self.display, *group_atom);
                    let group_name_cstr = std::ffi::CStr::from_ptr(group_name_cstr_ptr);
                    if let Ok(group_name) = group_name_cstr.to_str() {
                        layout_names.push(group_name.to_owned());
                    } else {
                        layout_names.push("None".to_owned());
                    }
                    XFree(group_name_cstr_ptr as *mut ::std::os::raw::c_void);
                }
            }
            XFree(symbol_names_cstr_ptr as *mut ::std::os::raw::c_void);

            XkbFreeKeyboard(keyboard, 0, 1);
        };
        layout_names
    }
}

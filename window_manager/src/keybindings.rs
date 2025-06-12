use std::{cmp::Reverse, process::Command};

use log::{error, trace};
use x11_bindings::{
    bindings::{
        XCB_MOD_MASK_1, XCB_MOD_MASK_2, XCB_MOD_MASK_4, XCB_MOD_MASK_CONTROL, XCB_MOD_MASK_LOCK,
        XCB_MOD_MASK_SHIFT, xcb_keycode_t, xcb_mod_mask_t,
    },
    connection::Connection,
};

use crate::{bar_message::UnixClients, config::Config, monitor::Monitor};

// extracted by running `xev` command line tool`
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keycodes {
    Escape = 9,
    Num_1 = 10,
    Num_2 = 11,
    Num_3 = 12,
    Num_4 = 13,
    Num_5 = 14,
    Num_6 = 15,
    Num_7 = 16,
    Num_8 = 17,
    Num_9 = 18,
    Num_0 = 19,
    minus = 20,
    equal = 21,
    BackSpace = 22,
    Tab = 23,
    q = 24,
    w = 25,
    e = 26,
    r = 27,
    t = 28,
    y = 29,
    u = 30,
    i = 31,
    o = 32,
    p = 33,
    bracketleft = 34,
    bracketright = 35,
    Return = 36,
    Control_L = 37,
    a = 38,
    s = 39,
    d = 40,
    f = 41,
    g = 42,
    h = 43,
    j = 44,
    k = 45,
    l = 46,
    semicolon = 47,
    apostrophe = 48,
    Shift_L = 50,
    backslash = 51,
    z = 52,
    x = 53,
    c = 54,
    v = 55,
    b = 56,
    n = 57,
    m = 58,
    comma = 59,
    period = 60,
    slash = 61,
    Shift_R = 62,
    Alt_L = 64,
    space = 65,
    F1 = 67,
    F2 = 68,
    F3 = 69,
    F4 = 70,
    F5 = 71,
    F6 = 72,
    F7 = 73,
    F8 = 74,
    F9 = 75,
    F10 = 76,
    KP_Home = 79,
    KP_Up = 80,
    KP_Prior = 81,
    KP_Subtract = 82,
    KP_Left = 83,
    KP_Begin = 84,
    KP_Right = 85,
    KP_Add = 86,
    KP_End = 87,
    KP_Down = 88,
    KP_Next = 89,
    KP_Insert = 90,
    KP_Delete = 91,
    less = 94,
    F11 = 95,
    F12 = 96,
    KP_Enter = 104,
    Control_R = 105,
    KP_Divide = 106,
    Print = 107,
    Alt_R = 108,
    Home = 110,
    Up = 111,
    Page_Up = 112,
    Left = 113,
    Right = 114,
    End = 115,
    Down = 116,
    Page_Down = 117,
    Insert = 118,
    Delete = 119,
    KP_Equal = 125,
    plusminus = 126,
    Pause = 127,
    KP_Decimal = 129,
    Super_L = 133,
    Super_R = 134,
    Menu = 135,
    Cancel = 136,
    Redo = 137,
    Undo = 139,
    Find = 144,
    Help = 146,
}

impl From<Keycodes> for u8 {
    fn from(value: Keycodes) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for Keycodes {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            9 => Ok(Keycodes::Escape),
            10 => Ok(Keycodes::Num_1),
            11 => Ok(Keycodes::Num_2),
            12 => Ok(Keycodes::Num_3),
            13 => Ok(Keycodes::Num_4),
            14 => Ok(Keycodes::Num_5),
            15 => Ok(Keycodes::Num_6),
            16 => Ok(Keycodes::Num_7),
            17 => Ok(Keycodes::Num_8),
            18 => Ok(Keycodes::Num_9),
            19 => Ok(Keycodes::Num_0),
            20 => Ok(Keycodes::minus),
            21 => Ok(Keycodes::equal),
            22 => Ok(Keycodes::BackSpace),
            23 => Ok(Keycodes::Tab),
            24 => Ok(Keycodes::q),
            25 => Ok(Keycodes::w),
            26 => Ok(Keycodes::e),
            27 => Ok(Keycodes::r),
            28 => Ok(Keycodes::t),
            29 => Ok(Keycodes::y),
            30 => Ok(Keycodes::u),
            31 => Ok(Keycodes::i),
            32 => Ok(Keycodes::o),
            33 => Ok(Keycodes::p),
            34 => Ok(Keycodes::bracketleft),
            35 => Ok(Keycodes::bracketright),
            36 => Ok(Keycodes::Return),
            37 => Ok(Keycodes::Control_L),
            38 => Ok(Keycodes::a),
            39 => Ok(Keycodes::s),
            40 => Ok(Keycodes::d),
            41 => Ok(Keycodes::f),
            42 => Ok(Keycodes::g),
            43 => Ok(Keycodes::h),
            44 => Ok(Keycodes::j),
            45 => Ok(Keycodes::k),
            46 => Ok(Keycodes::l),
            47 => Ok(Keycodes::semicolon),
            48 => Ok(Keycodes::apostrophe),
            50 => Ok(Keycodes::Shift_L),
            51 => Ok(Keycodes::backslash),
            52 => Ok(Keycodes::z),
            53 => Ok(Keycodes::x),
            54 => Ok(Keycodes::c),
            55 => Ok(Keycodes::v),
            56 => Ok(Keycodes::b),
            57 => Ok(Keycodes::n),
            58 => Ok(Keycodes::m),
            59 => Ok(Keycodes::comma),
            60 => Ok(Keycodes::period),
            61 => Ok(Keycodes::slash),
            62 => Ok(Keycodes::Shift_R),
            64 => Ok(Keycodes::Alt_L),
            65 => Ok(Keycodes::space),
            67 => Ok(Keycodes::F1),
            68 => Ok(Keycodes::F2),
            69 => Ok(Keycodes::F3),
            70 => Ok(Keycodes::F4),
            71 => Ok(Keycodes::F5),
            72 => Ok(Keycodes::F6),
            73 => Ok(Keycodes::F7),
            74 => Ok(Keycodes::F8),
            75 => Ok(Keycodes::F9),
            76 => Ok(Keycodes::F10),
            79 => Ok(Keycodes::KP_Home),
            80 => Ok(Keycodes::KP_Up),
            81 => Ok(Keycodes::KP_Prior),
            82 => Ok(Keycodes::KP_Subtract),
            83 => Ok(Keycodes::KP_Left),
            84 => Ok(Keycodes::KP_Begin),
            85 => Ok(Keycodes::KP_Right),
            86 => Ok(Keycodes::KP_Add),
            87 => Ok(Keycodes::KP_End),
            88 => Ok(Keycodes::KP_Down),
            89 => Ok(Keycodes::KP_Next),
            90 => Ok(Keycodes::KP_Insert),
            91 => Ok(Keycodes::KP_Delete),
            94 => Ok(Keycodes::less),
            95 => Ok(Keycodes::F11),
            96 => Ok(Keycodes::F12),
            104 => Ok(Keycodes::KP_Enter),
            105 => Ok(Keycodes::Control_R),
            106 => Ok(Keycodes::KP_Divide),
            107 => Ok(Keycodes::Print),
            108 => Ok(Keycodes::Alt_R),
            110 => Ok(Keycodes::Home),
            111 => Ok(Keycodes::Up),
            112 => Ok(Keycodes::Page_Up),
            113 => Ok(Keycodes::Left),
            114 => Ok(Keycodes::Right),
            115 => Ok(Keycodes::End),
            116 => Ok(Keycodes::Down),
            117 => Ok(Keycodes::Page_Down),
            118 => Ok(Keycodes::Insert),
            119 => Ok(Keycodes::Delete),
            125 => Ok(Keycodes::KP_Equal),
            126 => Ok(Keycodes::plusminus),
            127 => Ok(Keycodes::Pause),
            129 => Ok(Keycodes::KP_Decimal),
            133 => Ok(Keycodes::Super_L),
            134 => Ok(Keycodes::Super_R),
            135 => Ok(Keycodes::Menu),
            136 => Ok(Keycodes::Cancel),
            137 => Ok(Keycodes::Redo),
            139 => Ok(Keycodes::Undo),
            144 => Ok(Keycodes::Find),
            146 => Ok(Keycodes::Help),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for Keycodes {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let lower_case_value = value.to_lowercase();
        match lower_case_value.as_str() {
            "escape" => Ok(Keycodes::Escape),
            "1" => Ok(Keycodes::Num_1),
            "2" => Ok(Keycodes::Num_2),
            "3" => Ok(Keycodes::Num_3),
            "4" => Ok(Keycodes::Num_4),
            "5" => Ok(Keycodes::Num_5),
            "6" => Ok(Keycodes::Num_6),
            "7" => Ok(Keycodes::Num_7),
            "8" => Ok(Keycodes::Num_8),
            "9" => Ok(Keycodes::Num_9),
            "0" => Ok(Keycodes::Num_0),
            "minus" => Ok(Keycodes::minus),
            "equal" => Ok(Keycodes::equal),
            "backspace" => Ok(Keycodes::BackSpace),
            "tab" => Ok(Keycodes::Tab),
            "q" => Ok(Keycodes::q),
            "w" => Ok(Keycodes::w),
            "e" => Ok(Keycodes::e),
            "r" => Ok(Keycodes::r),
            "t" => Ok(Keycodes::t),
            "y" => Ok(Keycodes::y),
            "u" => Ok(Keycodes::u),
            "i" => Ok(Keycodes::i),
            "o" => Ok(Keycodes::o),
            "p" => Ok(Keycodes::p),
            "bracketleft" => Ok(Keycodes::bracketleft),
            "bracketright" => Ok(Keycodes::bracketright),
            "return" => Ok(Keycodes::Return),
            "enter" => Ok(Keycodes::Return),
            "control_l" => Ok(Keycodes::Control_L),
            "control" => Ok(Keycodes::Control_L),
            "ctrl" => Ok(Keycodes::Control_L),
            "a" => Ok(Keycodes::a),
            "s" => Ok(Keycodes::s),
            "d" => Ok(Keycodes::d),
            "f" => Ok(Keycodes::f),
            "g" => Ok(Keycodes::g),
            "h" => Ok(Keycodes::h),
            "j" => Ok(Keycodes::j),
            "k" => Ok(Keycodes::k),
            "l" => Ok(Keycodes::l),
            "semicolon" => Ok(Keycodes::semicolon),
            "apostrophe" => Ok(Keycodes::apostrophe),
            "shift_l" => Ok(Keycodes::Shift_L),
            "shift" => Ok(Keycodes::Shift_L),
            "backslash" => Ok(Keycodes::backslash),
            "z" => Ok(Keycodes::z),
            "x" => Ok(Keycodes::x),
            "c" => Ok(Keycodes::c),
            "v" => Ok(Keycodes::v),
            "b" => Ok(Keycodes::b),
            "n" => Ok(Keycodes::n),
            "m" => Ok(Keycodes::m),
            "comma" => Ok(Keycodes::comma),
            "period" => Ok(Keycodes::period),
            "slash" => Ok(Keycodes::slash),
            "shift_r" => Ok(Keycodes::Shift_R),
            "alt_l" => Ok(Keycodes::Alt_L),
            "alt" => Ok(Keycodes::Alt_L),
            "space" => Ok(Keycodes::space),
            "f1" => Ok(Keycodes::F1),
            "f2" => Ok(Keycodes::F2),
            "f3" => Ok(Keycodes::F3),
            "f4" => Ok(Keycodes::F4),
            "f5" => Ok(Keycodes::F5),
            "f6" => Ok(Keycodes::F6),
            "f7" => Ok(Keycodes::F7),
            "f8" => Ok(Keycodes::F8),
            "f9" => Ok(Keycodes::F9),
            "f10" => Ok(Keycodes::F10),
            "kp_home" => Ok(Keycodes::KP_Home),
            "kp_up" => Ok(Keycodes::KP_Up),
            "kp_prior" => Ok(Keycodes::KP_Prior),
            "kp_subtract" => Ok(Keycodes::KP_Subtract),
            "kp_left" => Ok(Keycodes::KP_Left),
            "kp_begin" => Ok(Keycodes::KP_Begin),
            "kp_right" => Ok(Keycodes::KP_Right),
            "kp_add" => Ok(Keycodes::KP_Add),
            "kp_end" => Ok(Keycodes::KP_End),
            "kp_down" => Ok(Keycodes::KP_Down),
            "kp_next" => Ok(Keycodes::KP_Next),
            "kp_insert" => Ok(Keycodes::KP_Insert),
            "kp_delete" => Ok(Keycodes::KP_Delete),
            "less" => Ok(Keycodes::less),
            "f11" => Ok(Keycodes::F11),
            "f12" => Ok(Keycodes::F12),
            "kp_enter" => Ok(Keycodes::KP_Enter),
            "control_r" => Ok(Keycodes::Control_R),
            "kp_divide" => Ok(Keycodes::KP_Divide),
            "print" => Ok(Keycodes::Print),
            "alt_r" => Ok(Keycodes::Alt_R),
            "home" => Ok(Keycodes::Home),
            "up" => Ok(Keycodes::Up),
            "page_up" => Ok(Keycodes::Page_Up),
            "left" => Ok(Keycodes::Left),
            "right" => Ok(Keycodes::Right),
            "end" => Ok(Keycodes::End),
            "down" => Ok(Keycodes::Down),
            "page_down" => Ok(Keycodes::Page_Down),
            "insert" => Ok(Keycodes::Insert),
            "delete" => Ok(Keycodes::Delete),
            "kp_equal" => Ok(Keycodes::KP_Equal),
            "plusminus" => Ok(Keycodes::plusminus),
            "pause" => Ok(Keycodes::Pause),
            "kp_decimal" => Ok(Keycodes::KP_Decimal),
            "super_l" => Ok(Keycodes::Super_L),
            "super" => Ok(Keycodes::Super_L),
            "super_r" => Ok(Keycodes::Super_R),
            "menu" => Ok(Keycodes::Menu),
            "cancel" => Ok(Keycodes::Cancel),
            "redo" => Ok(Keycodes::Redo),
            "undo" => Ok(Keycodes::Undo),
            "find" => Ok(Keycodes::Find),
            "help" => Ok(Keycodes::Help),
            _ => Err(()),
        }
    }
}

impl TryFrom<Keycodes> for xcb_mod_mask_t {
    type Error = ();

    fn try_from(value: Keycodes) -> Result<Self, Self::Error> {
        match value {
            Keycodes::Shift_L | Keycodes::Shift_R => Ok(XCB_MOD_MASK_SHIFT),
            Keycodes::Control_L | Keycodes::Control_R => Ok(XCB_MOD_MASK_CONTROL),
            Keycodes::Alt_L | Keycodes::Alt_R => Ok(XCB_MOD_MASK_1),
            Keycodes::Super_L | Keycodes::Super_R => Ok(XCB_MOD_MASK_4),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Dimension {
    Horizontal,
    Vertical,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum KeybindingAction {
    Exec(String),
    FocusWindow(Direction),
    MoveWindow(Direction),
    ResizeWindow(Dimension, i32),
    SwitchToWorkspace(u32),
    MoveFocusedWindowToWorkspace(u32),
    KillFocusedWindow,
    CenterFocusedWindow,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub struct Keybinding {
    pub modifiers: xcb_mod_mask_t,
    pub modifiers_count: usize,
    pub keycode: Keycodes,
    pub action: KeybindingAction,
}

pub fn execute_command_from_str(cmd_str: &str) {
    let mut parts = cmd_str.split_whitespace();
    if let Some(program_name) = parts.next() {
        let args = parts.collect::<Vec<_>>();
        match Command::new(program_name).args(args).spawn() {
            Ok(_) => trace!("Executed command successfully: {}", cmd_str),
            Err(err) => error!(
                "Failed to run command with error: {}, command: {}",
                err, cmd_str
            ),
        }
    } else {
        error!(
            "Failed to run command as it doesn't contain program name, command: {}",
            cmd_str
        );
    }
}

pub fn execute_command_from_str_wait(cmd_str: &str) {
    let mut parts = cmd_str.split_whitespace();
    if let Some(program_name) = parts.next() {
        let args = parts.collect::<Vec<_>>();
        match Command::new(program_name).args(args).status() {
            Ok(_) => trace!("Executed command successfully: {}", cmd_str),
            Err(err) => error!(
                "Failed to run command with error: {}, command: {}",
                err, cmd_str
            ),
        }
    } else {
        error!(
            "Failed to run command as it doesn't contain program name, command: {}",
            cmd_str
        );
    }
}

pub fn handle_key_press(
    keybindings: &Vec<Keybinding>,
    conn: &Connection,
    config: &Config,
    monitor: &mut Monitor,
    modifier: u16,
    keycode: xcb_keycode_t,
    unix_clients: &mut UnixClients,
) {
    trace!(
        "handle key press, modifier: {}, keycode: {}",
        modifier, keycode
    );
    let modifiers = modifier as u32 & !(XCB_MOD_MASK_2 | XCB_MOD_MASK_LOCK);
    if modifiers == 0 {
        error!("modifiers == 0");
        return;
    }
    let keycode = keycode as u8;
    for keybinding in keybindings {
        if (modifiers & keybinding.modifiers) == keybinding.modifiers
            && keycode == keybinding.keycode as u8
        {
            match &keybinding.action {
                KeybindingAction::Exec(cmd) => {
                    execute_command_from_str(cmd.as_str());
                }
                KeybindingAction::FocusWindow(direction) => {
                    monitor.handle_focus_window_change(conn, config, *direction, unix_clients);
                }
                KeybindingAction::MoveWindow(direction) => {
                    monitor.handle_move_window(conn, config, *direction, unix_clients);
                }
                KeybindingAction::ResizeWindow(dimension, size_change_pixels) => {
                    monitor.handle_resize_window(conn, config, *dimension, *size_change_pixels);
                }
                KeybindingAction::SwitchToWorkspace(workspace_id) => {
                    monitor.handle_switch_to_workspace(conn, config, *workspace_id, unix_clients);
                }
                KeybindingAction::MoveFocusedWindowToWorkspace(workspace_id) => {
                    monitor.handle_move_focused_window_to_workspace(
                        conn,
                        config,
                        *workspace_id,
                        config.switch_to_workspace_on_focused_window_moved,
                        unix_clients,
                    );
                }
                KeybindingAction::KillFocusedWindow => {
                    monitor.handle_kill_focused_window(conn, config);
                }
                KeybindingAction::CenterFocusedWindow => {
                    monitor.center_focused_window(conn, config);
                }
            };
            break;
        }
    }
}

pub fn keybindings_grab(keybindings: &Vec<Keybinding>, conn: &Connection) {
    for keybinding in keybindings {
        conn.grab_key(keybinding.modifiers, keybinding.keycode as u8);
        conn.grab_key(
            keybinding.modifiers | XCB_MOD_MASK_2,
            keybinding.keycode as u8,
        );
        conn.grab_key(
            keybinding.modifiers | XCB_MOD_MASK_LOCK,
            keybinding.keycode as u8,
        );
        conn.grab_key(
            keybinding.modifiers | XCB_MOD_MASK_2 | XCB_MOD_MASK_LOCK,
            keybinding.keycode as u8,
        );
    }
}

pub fn keybindings_ungrab(keybindings: &Vec<Keybinding>, conn: &Connection) {
    for keybinding in keybindings {
        conn.ungrab_key(keybinding.modifiers, keybinding.keycode as u8);
        conn.ungrab_key(
            keybinding.modifiers | XCB_MOD_MASK_2,
            keybinding.keycode as u8,
        );
        conn.ungrab_key(
            keybinding.modifiers | XCB_MOD_MASK_LOCK,
            keybinding.keycode as u8,
        );
        conn.ungrab_key(
            keybinding.modifiers | XCB_MOD_MASK_2 | XCB_MOD_MASK_LOCK,
            keybinding.keycode as u8,
        );
    }
}

pub fn keybindings_from_config(config: &Config) -> Vec<Keybinding> {
    let mut keybindings: Vec<Keybinding> = Vec::new();
    keybindings.reserve(config.keybindings.len());

    for keybinding_str in &config.keybindings {
        if let Some(keybinding) = keybinding_from_string(keybinding_str) {
            keybindings.push(keybinding);
        }
    }
    keybindings.sort_by_key(|e| Reverse(e.modifiers_count));

    keybindings
}

fn keybinding_from_string(keybinding_str: &str) -> Option<Keybinding> {
    let mut modifiers: xcb_mod_mask_t = 0;
    let mut modifiers_count: usize = 0;

    let mut parts = keybinding_str.trim().split_whitespace();
    if let Some(keys_comb_str) = parts.next() {
        let keys_comb = keys_comb_str.split('+').collect::<Vec<_>>();
        let count = &keys_comb.len();
        for mod_str in keys_comb.iter().take(count - 1) {
            if let Ok(mod_keycode) = Keycodes::try_from(*mod_str) {
                if let Ok(modifier) = xcb_mod_mask_t::try_from(mod_keycode) {
                    modifiers |= modifier;
                    modifiers_count += 1;
                } else {
                    error!("failed to extract modifier from keycode: {:?}", mod_keycode);
                    return None;
                }
            } else {
                error!(
                    "failed to extract keycode modifier from string: {}",
                    mod_str
                );
                return None;
            }
        }

        let keycode_str = keys_comb.last().unwrap();
        let keycode_maybe = Keycodes::try_from(*keycode_str);
        if keycode_maybe.is_err() {
            error!("failed to extract keycode from string: {}", keycode_str);
            return None;
        }
        if let Some(command) = parts.next() {
            match command {
                "exec" => {
                    let command_parts = parts.collect::<Vec<_>>().join(" ");
                    // trace!("command_parts: {}", command_parts);
                    return Some(Keybinding {
                        modifiers,
                        modifiers_count,
                        keycode: keycode_maybe.unwrap(),
                        action: KeybindingAction::Exec(command_parts),
                    });
                }
                "focus_window" => {
                    if let Some(focus_change_direction) = parts.next() {
                        match focus_change_direction {
                            "left" => {
                                return Some(Keybinding {
                                    modifiers,
                                    modifiers_count,
                                    keycode: keycode_maybe.unwrap(),
                                    action: KeybindingAction::FocusWindow(Direction::Left),
                                });
                            }
                            "right" => {
                                return Some(Keybinding {
                                    modifiers,
                                    modifiers_count,
                                    keycode: keycode_maybe.unwrap(),
                                    action: KeybindingAction::FocusWindow(Direction::Right),
                                });
                            }
                            "up" => {
                                return Some(Keybinding {
                                    modifiers,
                                    modifiers_count,
                                    keycode: keycode_maybe.unwrap(),
                                    action: KeybindingAction::FocusWindow(Direction::Up),
                                });
                            }
                            "down" => {
                                return Some(Keybinding {
                                    modifiers,
                                    modifiers_count,
                                    keycode: keycode_maybe.unwrap(),
                                    action: KeybindingAction::FocusWindow(Direction::Down),
                                });
                            }
                            _ => {
                                error!(
                                    "unknown focus change direction name: {}",
                                    focus_change_direction
                                );
                            }
                        }
                    } else {
                        error!(
                            "no direction supplied for focus window change command: {:?}",
                            parts
                        );
                    }
                }
                "move_window" => {
                    if let Some(move_window_direction) = parts.next() {
                        match move_window_direction {
                            "left" => {
                                return Some(Keybinding {
                                    modifiers,
                                    modifiers_count,
                                    keycode: keycode_maybe.unwrap(),
                                    action: KeybindingAction::MoveWindow(Direction::Left),
                                });
                            }
                            "right" => {
                                return Some(Keybinding {
                                    modifiers,
                                    modifiers_count,
                                    keycode: keycode_maybe.unwrap(),
                                    action: KeybindingAction::MoveWindow(Direction::Right),
                                });
                            }
                            "up" => {
                                return Some(Keybinding {
                                    modifiers,
                                    modifiers_count,
                                    keycode: keycode_maybe.unwrap(),
                                    action: KeybindingAction::MoveWindow(Direction::Up),
                                });
                            }
                            "down" => {
                                return Some(Keybinding {
                                    modifiers,
                                    modifiers_count,
                                    keycode: keycode_maybe.unwrap(),
                                    action: KeybindingAction::MoveWindow(Direction::Down),
                                });
                            }
                            _ => {
                                error!(
                                    "unknown move window direction name: {}",
                                    move_window_direction
                                );
                            }
                        }
                    } else {
                        error!("no direction supplied for move window command: {:?}", parts);
                    }
                }
                "window_size_change" => {
                    if let Some(resize_dimension) = parts.next() {
                        let maybe_size_change_str = parts.next();
                        if maybe_size_change_str.is_none() {
                            error!("no pixels in which size would be changed was specified");
                            return None;
                        }
                        let maybe_size_change = maybe_size_change_str.unwrap().parse::<i32>();
                        if maybe_size_change.is_err() {
                            error!(
                                "invalid size change pixels value: {}, error: {:?}",
                                maybe_size_change_str.unwrap(),
                                maybe_size_change.err()
                            );
                            return None;
                        }
                        let size_change_pixels = maybe_size_change.unwrap();
                        match resize_dimension {
                            "horizontal" => {
                                return Some(Keybinding {
                                    modifiers,
                                    modifiers_count,
                                    keycode: keycode_maybe.unwrap(),
                                    action: KeybindingAction::ResizeWindow(
                                        Dimension::Horizontal,
                                        size_change_pixels,
                                    ),
                                });
                            }
                            "vertical" => {
                                return Some(Keybinding {
                                    modifiers,
                                    modifiers_count,
                                    keycode: keycode_maybe.unwrap(),
                                    action: KeybindingAction::ResizeWindow(
                                        Dimension::Vertical,
                                        size_change_pixels,
                                    ),
                                });
                            }
                            _ => {
                                error!("unknown resize dimension name: {}", resize_dimension);
                            }
                        }
                    } else {
                        error!(
                            "no direction supplied for focus window change command: {:?}",
                            parts
                        );
                    }
                }
                "switch_to_workspace" => {
                    if let Some(workspace_id_str) = parts.next() {
                        match workspace_id_str.parse::<u32>() {
                            Ok(workspace_id) => {
                                return Some(Keybinding {
                                    modifiers,
                                    modifiers_count,
                                    keycode: keycode_maybe.unwrap(),
                                    action: KeybindingAction::SwitchToWorkspace(workspace_id),
                                });
                            }
                            Err(err) => {
                                error!(
                                    "invalid unsigned integer '{}' provided as a workspace id for switch to workspace command: {}, error: {:?}",
                                    workspace_id_str, command, err
                                );
                            }
                        }
                    } else {
                        error!(
                            "no workspace id provided for switch to workspace command: {}",
                            command
                        )
                    }
                }
                "move_focused_window_to_workspace" => {
                    if let Some(workspace_id_str) = parts.next() {
                        match workspace_id_str.parse::<u32>() {
                            Ok(workspace_id) => {
                                return Some(Keybinding {
                                    modifiers,
                                    modifiers_count,
                                    keycode: keycode_maybe.unwrap(),
                                    action: KeybindingAction::MoveFocusedWindowToWorkspace(
                                        workspace_id,
                                    ),
                                });
                            }
                            Err(err) => {
                                error!(
                                    "invalid unsigned integer '{}' provided as a workspace id for move focused window to workspace command: {}, error: {:?}",
                                    workspace_id_str, command, err
                                );
                            }
                        }
                    } else {
                        error!(
                            "no workspace id provided for move focused window to workspace command: {}",
                            command
                        )
                    }
                }
                "kill_focused_window" => {
                    return Some(Keybinding {
                        modifiers,
                        modifiers_count,
                        keycode: keycode_maybe.unwrap(),
                        action: KeybindingAction::KillFocusedWindow,
                    });
                }
                "center_focused_window" => {
                    return Some(Keybinding {
                        modifiers,
                        modifiers_count,
                        keycode: keycode_maybe.unwrap(),
                        action: KeybindingAction::CenterFocusedWindow,
                    });
                }
                _ => error!("no command matching string: {}", command),
            }
        } else {
            error!("failed to find command in keybinding: {}", keybinding_str);
        }
    }
    return None;
}

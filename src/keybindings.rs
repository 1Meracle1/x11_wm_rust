use std::{
    process::{Command, Stdio},
    str::FromStr,
};

use log::{debug, error, info};

use crate::config::Config;
use xcb::x;

#[derive(Debug)]
pub enum Action {
    Exec(String),
}

impl Action {
    fn execute(&self) {
        match self {
            Action::Exec(command) => {
                match Command::new(command.clone())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                {
                    Ok(_) => info!("Successfully executed command: '{command}'"),
                    Err(error) => {
                        error!("Failed to executed command: '{command}', error: '{error}'")
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Keybinding {
    pub modifiers: x::ModMask,
    pub key_name: Keycodes,
    pub action: Action,
    pub weight: u32,
}

pub struct KeyEventsHandler {
    binds: Vec<Keybinding>,
}

impl KeyEventsHandler {
    pub fn new(conn: &xcb::Connection, root_window: x::Window, config: &Config) -> Self {
        let mut binds = Vec::new();
        'outer: for bind in &config.keybindings {
            let keys = bind.keys_combination.split('+');

            let mut modifiers = x::ModMask::empty();
            let mut key_name = Keycodes::None;
            let mut weight: u32 = 0;
            for key in keys {
                match Keycodes::from_str(&key) {
                    Ok(key) => {
                        if let Some(modifier) = Into::<Option<x::ModMask>>::into(key) {
                            modifiers = modifiers.union(modifier);
                            weight += 1;
                        } else {
                            key_name = key;
                            weight += 1;
                            break;
                        }
                    }
                    Err(err) => {
                        error!(
                            "Invalid binding, error parsing modifiers: '{err}', binding: '{:#?}'",
                            bind
                        );
                        continue 'outer;
                    }
                };
            }
            if modifiers.is_empty() || key_name == Keycodes::None {
                error!(
                    "Invalid binding: no modifier/keys supplied, binding: {:#?}",
                    bind
                );
                continue;
            }

            match bind.action.as_str() {
                "exec" => {
                    if bind.arguments.len() != 1 {
                        error!(
                            "Arguments len is {}, invalid binding found: {:#?}",
                            bind.arguments.len(),
                            bind
                        );
                        continue;
                    }

                    conn.send_request(&x::GrabKey {
                        owner_events: false,
                        grab_window: root_window,
                        modifiers,
                        key: key_name as u8,
                        pointer_mode: x::GrabMode::Async,
                        keyboard_mode: x::GrabMode::Async,
                    });

                    binds.push(Keybinding {
                        modifiers,
                        key_name,
                        action: Action::Exec(bind.arguments.first().unwrap().clone()),
                        weight,
                    });
                }
                _ => {}
            }
        }

        binds.sort_by(|lhs, rhs| lhs.weight.cmp(&rhs.weight));
        debug!("binds: {:#?}", binds);

        Self { binds }
    }

    pub fn handle_key_press(&self, event: x::KeyPressEvent) {
        let modifiers = event
            .state()
            .difference(x::KeyButMask::LOCK | x::KeyButMask::MOD2);
        let modifiers = key_into_mod_mask(modifiers);
        for bind in &self.binds {
            if bind.modifiers.contains(modifiers) && event.detail() == bind.key_name as u8 {
                bind.action.execute();
                break;
            }
        }
    }
}

fn key_into_mod_mask(key_mask: x::KeyButMask) -> x::ModMask {
    let mut mod_mask = x::ModMask::empty();

    if key_mask.contains(x::KeyButMask::SHIFT) {
        mod_mask = mod_mask.union(x::ModMask::SHIFT);
    }
    if key_mask.contains(x::KeyButMask::LOCK) {
        mod_mask = mod_mask.union(x::ModMask::LOCK);
    }
    if key_mask.contains(x::KeyButMask::CONTROL) {
        mod_mask = mod_mask.union(x::ModMask::CONTROL);
    }
    if key_mask.contains(x::KeyButMask::MOD1) {
        mod_mask = mod_mask.union(x::ModMask::N1);
    }
    if key_mask.contains(x::KeyButMask::MOD2) {
        mod_mask = mod_mask.union(x::ModMask::N2);
    }
    if key_mask.contains(x::KeyButMask::MOD3) {
        mod_mask = mod_mask.union(x::ModMask::N3);
    }
    if key_mask.contains(x::KeyButMask::MOD4) {
        mod_mask = mod_mask.union(x::ModMask::N4);
    }
    if key_mask.contains(x::KeyButMask::MOD5) {
        mod_mask = mod_mask.union(x::ModMask::N5);
    }

    mod_mask
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Keycodes {
    None = 0,
    Escape = 9,
    _1 = 10,
    _2 = 11,
    _3 = 12,
    _4 = 13,
    _5 = 14,
    _6 = 15,
    _7 = 16,
    _8 = 17,
    _9 = 18,
    _0 = 19,
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
    grave = 49,
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
    KP_Multiply = 63,
    Alt_L = 64,
    space = 65,
    Caps_Lock = 66,
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
    Num_Lock = 77,
    Scroll_Lock = 78,
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
    ISO_Level3_Shift = 92,
    less = 94,
    F11 = 95,
    F12 = 96,
    Katakana = 98,
    Hiragana = 99,
    Henkan_Mode = 100,
    Hiragana_Katakana = 101,
    Muhenkan = 102,
    KP_Enter = 104,
    Control_R = 105,
    KP_Divide = 106,
    Print = 107,
    Alt_R = 108,
    Linefeed = 109,
    Home = 110,
    Up = 111,
    Prior = 112,
    Left = 113,
    Right = 114,
    End = 115,
    Down = 116,
    Next = 117,
    Insert = 118,
    Delete = 119,
    XF86AudioMute = 121,
    XF86AudioLowerVolume = 122,
    XF86AudioRaiseVolume = 123,
    XF86PowerOff = 124,
    KP_Equal = 125,
    plusminus = 126,
    Pause = 127,
    XF86LaunchA = 128,
    KP_Decimal = 129,
    Hangul = 130,
    Hangul_Hanja = 131,
    Super_L = 133,
    Super_R = 134,
    Menu = 135,
    Cancel = 136,
    Redo = 137,
    SunProps = 138,
    Undo = 139,
    SunFront = 140,
    XF86Copy = 141,
    XF86Open = 142,
    XF86Paste = 143,
    Find = 144,
    XF86Cut = 145,
    Help = 146,
    XF86MenuKB = 147,
    XF86Calculator = 148,
    XF86Sleep = 150,
    XF86WakeUp = 151,
    XF86Explorer = 152,
    XF86Send = 153,
    XF86Xfer = 155,
    XF86Launch1 = 156,
    XF86Launch2 = 157,
    XF86WWW = 158,
    XF86DOS = 159,
    XF86ScreenSaver = 160,
    XF86RotateWindows = 161,
    XF86TaskPane = 162,
    XF86Mail = 163,
    XF86Favorites = 164,
    XF86MyComputer = 165,
    XF86Back = 166,
    XF86Forward = 167,
    XF86Eject = 169,
    XF86AudioNext = 171,
    XF86AudioPlay = 172,
    XF86AudioPrev = 173,
    XF86AudioStop = 174,
    XF86AudioRecord = 175,
    XF86AudioRewind = 176,
    XF86Phone = 177,
    XF86Tools = 179,
    XF86HomePage = 180,
    XF86Reload = 181,
    XF86Close = 182,
    XF86ScrollUp = 185,
    XF86ScrollDown = 186,
    parenleft = 187,
    parenright = 188,
    XF86New = 189,
    XF86Launch5 = 192,
    XF86Launch6 = 193,
    XF86Launch7 = 194,
    XF86Launch8 = 195,
    XF86Launch9 = 196,
    XF86AudioMicMute = 198,
    XF86TouchpadToggle = 199,
    XF86TouchpadOn = 200,
    XF86TouchpadOff = 201,
    ISO_Level5_Shift = 203,
    NoSymbol = 204,
    XF86AudioPause = 209,
    XF86Launch3 = 210,
    XF86Launch4 = 211,
    XF86LaunchB = 212,
    XF86Suspend = 213,
    XF86AudioForward = 216,
    XF86WebCam = 220,
    XF86AudioPreset = 221,
    XF86Messenger = 224,
    XF86Search = 225,
    XF86Go = 226,
    XF86Finance = 227,
    XF86Game = 228,
    XF86Shop = 229,
    XF86MonBrightnessDown = 232,
    XF86MonBrightnessUp = 233,
    XF86AudioMedia = 234,
    XF86Display = 235,
    XF86KbdLightOnOff = 236,
    XF86KbdBrightnessDown = 237,
    XF86KbdBrightnessUp = 238,
    XF86Reply = 240,
    XF86MailForward = 241,
    XF86Save = 242,
    XF86Documents = 243,
    XF86Battery = 244,
    XF86Bluetooth = 245,
    XF86WLAN = 246,
    XF86UWB = 247,
    XF86Next_VMode = 249,
    XF86Prev_VMode = 250,
    XF86MonBrightnessCycle = 251,
    XF86BrightnessAuto = 252,
    XF86DisplayOff = 253,
    XF86WWAN = 254,
    XF86RFKill = 255,
}

impl FromStr for Keycodes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Alt" => Ok(Keycodes::Alt_L),
            "Escape" => Ok(Keycodes::Escape),
            "1" => Ok(Keycodes::_1),
            "2" => Ok(Keycodes::_2),
            "3" => Ok(Keycodes::_3),
            "4" => Ok(Keycodes::_4),
            "5" => Ok(Keycodes::_5),
            "6" => Ok(Keycodes::_6),
            "7" => Ok(Keycodes::_7),
            "8" => Ok(Keycodes::_8),
            "9" => Ok(Keycodes::_9),
            "0" => Ok(Keycodes::_0),
            "minus" => Ok(Keycodes::minus),
            "equal" => Ok(Keycodes::equal),
            "BackSpace" => Ok(Keycodes::BackSpace),
            "Tab" => Ok(Keycodes::Tab),
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
            "Q" => Ok(Keycodes::q),
            "W" => Ok(Keycodes::w),
            "E" => Ok(Keycodes::e),
            "R" => Ok(Keycodes::r),
            "T" => Ok(Keycodes::t),
            "Y" => Ok(Keycodes::y),
            "U" => Ok(Keycodes::u),
            "I" => Ok(Keycodes::i),
            "O" => Ok(Keycodes::o),
            "P" => Ok(Keycodes::p),
            "bracketleft" => Ok(Keycodes::bracketleft),
            "bracketright" => Ok(Keycodes::bracketright),
            "Return" => Ok(Keycodes::Return),
            "Control_L" => Ok(Keycodes::Control_L),
            "Control" => Ok(Keycodes::Control_L),
            "a" => Ok(Keycodes::a),
            "s" => Ok(Keycodes::s),
            "d" => Ok(Keycodes::d),
            "f" => Ok(Keycodes::f),
            "g" => Ok(Keycodes::g),
            "h" => Ok(Keycodes::h),
            "j" => Ok(Keycodes::j),
            "k" => Ok(Keycodes::k),
            "l" => Ok(Keycodes::l),
            "A" => Ok(Keycodes::a),
            "S" => Ok(Keycodes::s),
            "D" => Ok(Keycodes::d),
            "F" => Ok(Keycodes::f),
            "G" => Ok(Keycodes::g),
            "H" => Ok(Keycodes::h),
            "J" => Ok(Keycodes::j),
            "K" => Ok(Keycodes::k),
            "L" => Ok(Keycodes::l),
            "semicolon" => Ok(Keycodes::semicolon),
            "apostrophe" => Ok(Keycodes::apostrophe),
            "grave" => Ok(Keycodes::grave),
            "Shift_L" => Ok(Keycodes::Shift_L),
            "Shift" => Ok(Keycodes::Shift_L),
            "backslash" => Ok(Keycodes::backslash),
            "z" => Ok(Keycodes::z),
            "x" => Ok(Keycodes::x),
            "c" => Ok(Keycodes::c),
            "v" => Ok(Keycodes::v),
            "b" => Ok(Keycodes::b),
            "n" => Ok(Keycodes::n),
            "m" => Ok(Keycodes::m),
            "Z" => Ok(Keycodes::z),
            "X" => Ok(Keycodes::x),
            "C" => Ok(Keycodes::c),
            "V" => Ok(Keycodes::v),
            "B" => Ok(Keycodes::b),
            "N" => Ok(Keycodes::n),
            "M" => Ok(Keycodes::m),
            "comma" => Ok(Keycodes::comma),
            "period" => Ok(Keycodes::period),
            "slash" => Ok(Keycodes::slash),
            "Shift_R" => Ok(Keycodes::Shift_R),
            "KP_Multiply" => Ok(Keycodes::KP_Multiply),
            "Alt_L" => Ok(Keycodes::Alt_L),
            "space" => Ok(Keycodes::space),
            "Caps_Lock" => Ok(Keycodes::Caps_Lock),
            "F1" => Ok(Keycodes::F1),
            "F2" => Ok(Keycodes::F2),
            "F3" => Ok(Keycodes::F3),
            "F4" => Ok(Keycodes::F4),
            "F5" => Ok(Keycodes::F5),
            "F6" => Ok(Keycodes::F6),
            "F7" => Ok(Keycodes::F7),
            "F8" => Ok(Keycodes::F8),
            "F9" => Ok(Keycodes::F9),
            "F10" => Ok(Keycodes::F10),
            "Num_Lock" => Ok(Keycodes::Num_Lock),
            "Scroll_Lock" => Ok(Keycodes::Scroll_Lock),
            "KP_Home" => Ok(Keycodes::KP_Home),
            "KP_Up" => Ok(Keycodes::KP_Up),
            "KP_Prior" => Ok(Keycodes::KP_Prior),
            "KP_Subtract" => Ok(Keycodes::KP_Subtract),
            "KP_Left" => Ok(Keycodes::KP_Left),
            "KP_Begin" => Ok(Keycodes::KP_Begin),
            "KP_Right" => Ok(Keycodes::KP_Right),
            "KP_Add" => Ok(Keycodes::KP_Add),
            "KP_End" => Ok(Keycodes::KP_End),
            "KP_Down" => Ok(Keycodes::KP_Down),
            "KP_Next" => Ok(Keycodes::KP_Next),
            "KP_Insert" => Ok(Keycodes::KP_Insert),
            "KP_Delete" => Ok(Keycodes::KP_Delete),
            "ISO_Level3_Shift" => Ok(Keycodes::ISO_Level3_Shift),
            "less" => Ok(Keycodes::less),
            "F11" => Ok(Keycodes::F11),
            "F12" => Ok(Keycodes::F12),
            "Katakana" => Ok(Keycodes::Katakana),
            "Hiragana" => Ok(Keycodes::Hiragana),
            "Henkan_Mode" => Ok(Keycodes::Henkan_Mode),
            "Hiragana_Katakana" => Ok(Keycodes::Hiragana_Katakana),
            "Muhenkan" => Ok(Keycodes::Muhenkan),
            "KP_Enter" => Ok(Keycodes::KP_Enter),
            "Control_R" => Ok(Keycodes::Control_R),
            "KP_Divide" => Ok(Keycodes::KP_Divide),
            "Print" => Ok(Keycodes::Print),
            "Alt_R" => Ok(Keycodes::Alt_R),
            "Linefeed" => Ok(Keycodes::Linefeed),
            "Home" => Ok(Keycodes::Home),
            "Up" => Ok(Keycodes::Up),
            "Prior" => Ok(Keycodes::Prior),
            "Left" => Ok(Keycodes::Left),
            "Right" => Ok(Keycodes::Right),
            "End" => Ok(Keycodes::End),
            "Down" => Ok(Keycodes::Down),
            "Next" => Ok(Keycodes::Next),
            "Insert" => Ok(Keycodes::Insert),
            "Delete" => Ok(Keycodes::Delete),
            "XF86AudioMute" => Ok(Keycodes::XF86AudioMute),
            "XF86AudioLowerVolume" => Ok(Keycodes::XF86AudioLowerVolume),
            "XF86AudioRaiseVolume" => Ok(Keycodes::XF86AudioRaiseVolume),
            "XF86PowerOff" => Ok(Keycodes::XF86PowerOff),
            "KP_Equal" => Ok(Keycodes::KP_Equal),
            "plusminus" => Ok(Keycodes::plusminus),
            "Pause" => Ok(Keycodes::Pause),
            "XF86LaunchA" => Ok(Keycodes::XF86LaunchA),
            "KP_Decimal" => Ok(Keycodes::KP_Decimal),
            "Hangul" => Ok(Keycodes::Hangul),
            "Hangul_Hanja" => Ok(Keycodes::Hangul_Hanja),
            "Super_L" => Ok(Keycodes::Super_L),
            "Super_R" => Ok(Keycodes::Super_R),
            "Menu" => Ok(Keycodes::Menu),
            "Cancel" => Ok(Keycodes::Cancel),
            "Redo" => Ok(Keycodes::Redo),
            "SunProps" => Ok(Keycodes::SunProps),
            "Undo" => Ok(Keycodes::Undo),
            "SunFront" => Ok(Keycodes::SunFront),
            "XF86Copy" => Ok(Keycodes::XF86Copy),
            "XF86Open" => Ok(Keycodes::XF86Open),
            "XF86Paste" => Ok(Keycodes::XF86Paste),
            "Find" => Ok(Keycodes::Find),
            "XF86Cut" => Ok(Keycodes::XF86Cut),
            "Help" => Ok(Keycodes::Help),
            "XF86MenuKB" => Ok(Keycodes::XF86MenuKB),
            "XF86Calculator" => Ok(Keycodes::XF86Calculator),
            "XF86Sleep" => Ok(Keycodes::XF86Sleep),
            "XF86WakeUp" => Ok(Keycodes::XF86WakeUp),
            "XF86Explorer" => Ok(Keycodes::XF86Explorer),
            "XF86Send" => Ok(Keycodes::XF86Send),
            "XF86Xfer" => Ok(Keycodes::XF86Xfer),
            "XF86Launch1" => Ok(Keycodes::XF86Launch1),
            "XF86Launch2" => Ok(Keycodes::XF86Launch2),
            "XF86WWW" => Ok(Keycodes::XF86WWW),
            "XF86DOS" => Ok(Keycodes::XF86DOS),
            "XF86ScreenSaver" => Ok(Keycodes::XF86ScreenSaver),
            "XF86RotateWindows" => Ok(Keycodes::XF86RotateWindows),
            "XF86TaskPane" => Ok(Keycodes::XF86TaskPane),
            "XF86Mail" => Ok(Keycodes::XF86Mail),
            "XF86Favorites" => Ok(Keycodes::XF86Favorites),
            "XF86MyComputer" => Ok(Keycodes::XF86MyComputer),
            "XF86Back" => Ok(Keycodes::XF86Back),
            "XF86Forward" => Ok(Keycodes::XF86Forward),
            "XF86Eject" => Ok(Keycodes::XF86Eject),
            "XF86AudioNext" => Ok(Keycodes::XF86AudioNext),
            "XF86AudioPlay" => Ok(Keycodes::XF86AudioPlay),
            "XF86AudioPrev" => Ok(Keycodes::XF86AudioPrev),
            "XF86AudioStop" => Ok(Keycodes::XF86AudioStop),
            "XF86AudioRecord" => Ok(Keycodes::XF86AudioRecord),
            "XF86AudioRewind" => Ok(Keycodes::XF86AudioRewind),
            "XF86Phone" => Ok(Keycodes::XF86Phone),
            "XF86Tools" => Ok(Keycodes::XF86Tools),
            "XF86HomePage" => Ok(Keycodes::XF86HomePage),
            "XF86Reload" => Ok(Keycodes::XF86Reload),
            "XF86Close" => Ok(Keycodes::XF86Close),
            "XF86ScrollUp" => Ok(Keycodes::XF86ScrollUp),
            "XF86ScrollDown" => Ok(Keycodes::XF86ScrollDown),
            "parenleft" => Ok(Keycodes::parenleft),
            "parenright" => Ok(Keycodes::parenright),
            "XF86New" => Ok(Keycodes::XF86New),
            "XF86Launch5" => Ok(Keycodes::XF86Launch5),
            "XF86Launch6" => Ok(Keycodes::XF86Launch6),
            "XF86Launch7" => Ok(Keycodes::XF86Launch7),
            "XF86Launch8" => Ok(Keycodes::XF86Launch8),
            "XF86Launch9" => Ok(Keycodes::XF86Launch9),
            "XF86AudioMicMute" => Ok(Keycodes::XF86AudioMicMute),
            "XF86TouchpadToggle" => Ok(Keycodes::XF86TouchpadToggle),
            "XF86TouchpadOn" => Ok(Keycodes::XF86TouchpadOn),
            "XF86TouchpadOff" => Ok(Keycodes::XF86TouchpadOff),
            "ISO_Level5_Shift" => Ok(Keycodes::ISO_Level5_Shift),
            "NoSymbol" => Ok(Keycodes::NoSymbol),
            "XF86AudioPause" => Ok(Keycodes::XF86AudioPause),
            "XF86Launch3" => Ok(Keycodes::XF86Launch3),
            "XF86Launch4" => Ok(Keycodes::XF86Launch4),
            "XF86LaunchB" => Ok(Keycodes::XF86LaunchB),
            "XF86Suspend" => Ok(Keycodes::XF86Suspend),
            "XF86AudioForward" => Ok(Keycodes::XF86AudioForward),
            "XF86WebCam" => Ok(Keycodes::XF86WebCam),
            "XF86AudioPreset" => Ok(Keycodes::XF86AudioPreset),
            "XF86Messenger" => Ok(Keycodes::XF86Messenger),
            "XF86Search" => Ok(Keycodes::XF86Search),
            "XF86Go" => Ok(Keycodes::XF86Go),
            "XF86Finance" => Ok(Keycodes::XF86Finance),
            "XF86Game" => Ok(Keycodes::XF86Game),
            "XF86Shop" => Ok(Keycodes::XF86Shop),
            "XF86MonBrightnessDown" => Ok(Keycodes::XF86MonBrightnessDown),
            "XF86MonBrightnessUp" => Ok(Keycodes::XF86MonBrightnessUp),
            "XF86AudioMedia" => Ok(Keycodes::XF86AudioMedia),
            "XF86Display" => Ok(Keycodes::XF86Display),
            "XF86KbdLightOnOff" => Ok(Keycodes::XF86KbdLightOnOff),
            "XF86KbdBrightnessDown" => Ok(Keycodes::XF86KbdBrightnessDown),
            "XF86KbdBrightnessUp" => Ok(Keycodes::XF86KbdBrightnessUp),
            "XF86Reply" => Ok(Keycodes::XF86Reply),
            "XF86MailForward" => Ok(Keycodes::XF86MailForward),
            "XF86Save" => Ok(Keycodes::XF86Save),
            "XF86Documents" => Ok(Keycodes::XF86Documents),
            "XF86Battery" => Ok(Keycodes::XF86Battery),
            "XF86Bluetooth" => Ok(Keycodes::XF86Bluetooth),
            "XF86WLAN" => Ok(Keycodes::XF86WLAN),
            "XF86UWB" => Ok(Keycodes::XF86UWB),
            "XF86Next_VMode" => Ok(Keycodes::XF86Next_VMode),
            "XF86Prev_VMode" => Ok(Keycodes::XF86Prev_VMode),
            "XF86MonBrightnessCycle" => Ok(Keycodes::XF86MonBrightnessCycle),
            "XF86BrightnessAuto" => Ok(Keycodes::XF86BrightnessAuto),
            "XF86DisplayOff" => Ok(Keycodes::XF86DisplayOff),
            "XF86WWAN" => Ok(Keycodes::XF86WWAN),
            "XF86RFKill" => Ok(Keycodes::XF86RFKill),
            _ => Err(format!("No matching keycode for '{s}'_ ")),
        }
    }
}

impl Into<Option<x::ModMask>> for Keycodes {
    fn into(self) -> Option<x::ModMask> {
        match self {
            Self::Shift_L => Some(x::ModMask::SHIFT),
            Self::Shift_R => Some(x::ModMask::SHIFT),
            Self::Caps_Lock => Some(x::ModMask::LOCK),
            Self::Control_L => Some(x::ModMask::CONTROL),
            Self::Control_R => Some(x::ModMask::CONTROL),
            Self::Alt_L => Some(x::ModMask::N1),
            Self::Alt_R => Some(x::ModMask::N5),
            Self::Num_Lock => Some(x::ModMask::N2),
            Self::Super_L => Some(x::ModMask::N4),
            Self::Super_R => Some(x::ModMask::N4),
            _ => None,
        }
    }
}

impl Default for Keycodes {
    fn default() -> Self {
        Keycodes::None
    }
}

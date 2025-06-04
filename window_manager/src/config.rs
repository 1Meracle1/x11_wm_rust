use std::{fs, io, path::Path};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub startup_commands: Vec<String>,
    pub keybindings: Vec<String>,
    pub minimum_width_tiling: u32,
    pub minimum_height_tiling: u32,
    pub default_screen_width_percent_tiling: f64,
    pub outer_gap_horiz: u32,
    pub outer_gap_vert: u32,
    pub inner_gap: u32,
    pub border_size: u32,
    pub border_color_inactive_str: Option<String>,
    pub border_color_active_str: Option<String>,
    pub border_color_inactive_int: Option<u32>,
    pub border_color_active_int: Option<u32>,
    pub switch_to_workspace_on_focused_window_moved: bool,
    pub override_to_floating: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ConfigErrors {
    FileNotFound(io::Error),
    TomlParseError(toml::de::Error),
    ValidationError(String),
}

impl Config {
    pub fn new(path: &str) -> Result<Self, ConfigErrors> {
        match fs::read_to_string(Path::new(path)) {
            Ok(config_data) => match toml::from_str::<Config>(&config_data) {
                Ok(mut config) => {
                    if config.border_color_inactive_str.is_none()
                        && config.border_color_inactive_int.is_none()
                    {
                        return Err(ConfigErrors::ValidationError(
                            "border_color_inactive is not populated".to_string(),
                        ));
                    }
                    if config.border_color_active_str.is_none()
                        && config.border_color_active_int.is_none()
                    {
                        return Err(ConfigErrors::ValidationError(
                            "border_color_active is not populated".to_string(),
                        ));
                    }
                    if config.border_color_inactive_int.is_none() {
                        let color_str = config.border_color_inactive_str.as_ref().unwrap();
                        if let Some(color_int) = Self::try_color_from_str(&color_str) {
                            config.border_color_inactive_int = Some(color_int);
                        } else {
                            return Err(ConfigErrors::ValidationError(format!(
                                "border_color_inactive_str contains invalid hex value: {}",
                                color_str
                            )));
                        }
                    }
                    if config.border_color_active_int.is_none() {
                        let color_str = config.border_color_active_str.as_ref().unwrap();
                        if let Some(color_int) = Self::try_color_from_str(&color_str) {
                            config.border_color_active_int = Some(color_int);
                        } else {
                            return Err(ConfigErrors::ValidationError(format!(
                                "border_color_active_str contains invalid hex value: {}",
                                color_str
                            )));
                        }
                    }
                    Ok(config)
                }
                Err(err) => Err(ConfigErrors::TomlParseError(err)),
            },
            Err(err) => Err(ConfigErrors::FileNotFound(err)),
        }
    }

    fn try_color_from_str(color_str: &str) -> Option<u32> {
        // take only first 6 characters as xcb_change_window_attributes supports only 24-bit color range
        let hex = color_str.trim_start_matches('#').chars().take(6).collect::<String>();
        if let Ok(res) = u32::from_str_radix(&hex, 16) {
            Some(res | (0xff << 24))
        } else {
            None
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            startup_commands: Default::default(),
            keybindings: [
                "Alt+Left            focus_window left",
                "Alt+Right           focus_window right",
                "Alt+Up              focus_window up",
                "Alt+Down            focus_window down",
            
                "Alt+H               focus_window left",
                "Alt+L               focus_window right",
                "Alt+K               focus_window up",
                "Alt+J               focus_window down",
            
                "Alt+Left            focus_window left",
                "Alt+Right           focus_window right",
                "Alt+Up              focus_window up",
                "Alt+Down            focus_window down",
            
                "Alt+Ctrl+H          move_window left  15",
                "Alt+Ctrl+L          move_window right 15",
                "Alt+Ctrl+K          move_window up    15",
                "Alt+Ctrl+J          move_window down  15",
            
                "Alt+Shift+H         window_size_change horizontal  15",
                "Alt+Shift+L         window_size_change horizontal  15",
                "Alt+Shift+J         window_size_change vertical    15",
                "Alt+Shift+K         window_size_change vertical    15",
            
                "Alt+Enter           exec alacritty",
                "Alt+D               exec cmake_debug_build/testbed_window -window-type=docked -docked-location=bottom",
                "Alt+T               exec cmake_debug_build/testbed_window -window-type=docked -docked-location=top",
            
                "Alt+D               exec dmenu_run -i -nb '#191919' -nf 'orange' -sb 'orange' -sf '#191919' -fn 'NotoMonoRegular:bold:pixelsize=14'",
            
                "Alt+Q               kill_focused_window",
            
                "Alt+Shift+R         restart_window_manager",
            
                "Alt+Shift+E         exec prompt_exit_window_manager",
                "Alt+Shift+Q         exit_window_manager",
            
                "Alt+1               switch_to_workspace 1",
                "Alt+2               switch_to_workspace 2",
                "Alt+3               switch_to_workspace 3",
                "Alt+4               switch_to_workspace 4",
                "Alt+5               switch_to_workspace 5",
                "Alt+6               switch_to_workspace 6",
                "Alt+7               switch_to_workspace 7",
                "Alt+8               switch_to_workspace 8",
                "Alt+9               switch_to_workspace 9",
            
                "Alt+Shift+1         move_focused_window_to_workspace 1",
                "Alt+Shift+2         move_focused_window_to_workspace 2",
                "Alt+Shift+3         move_focused_window_to_workspace 3",
                "Alt+Shift+4         move_focused_window_to_workspace 4",
                "Alt+Shift+5         move_focused_window_to_workspace 5",
                "Alt+Shift+6         move_focused_window_to_workspace 6",
                "Alt+Shift+7         move_focused_window_to_workspace 7",
                "Alt+Shift+8         move_focused_window_to_workspace 8",
                "Alt+Shift+9         move_focused_window_to_workspace 9",
            ].iter().map(|e|e.to_string()).collect(),
            minimum_width_tiling: 10,
            minimum_height_tiling: 10,
            default_screen_width_percent_tiling: 0.3333,
            outer_gap_horiz: 10,
            outer_gap_vert: 10,
            inner_gap: 5,
            border_size: 3,
            border_color_inactive_str: None,
            border_color_active_str: None,
            border_color_inactive_int: Self::try_color_from_str("#2b2b29"),
            border_color_active_int: Self::try_color_from_str("#a38b43"),
            switch_to_workspace_on_focused_window_moved: false,
            override_to_floating: [""].iter().map(|e|e.to_string()).collect(),
        }
    }
}

#[allow(dead_code)]
#[inline]
fn create_color_with_alpha(red: u8, green: u8, blue: u8, alpha: u8) -> u32 {
    ((alpha as u32) << 24) | ((red as u32) << 16) | ((green as u32) << 8) | (blue as u32)
}
use std::{fs, path::Path};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct KeybindingsConfig {
    pub keys_combination: String,
    pub action: String,
    pub arguments: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct WindowBorderConfig {
    pub size: u16,
    pub color_inactive: String,
    pub color_active: String,
    pub color_inactive_u32: Option<u32>,
    pub color_active_u32: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub struct WindowConfig {
    pub border: WindowBorderConfig,
    pub minimum_width_tiling: u16,
    pub default_width_tiling: f32,
}

#[derive(Deserialize, Debug)]
pub struct WorkspacesConfig {
    pub count: u16,
    pub outer_gap_horiz: u16,
    pub outer_gap_vert: u16,
    pub inner_gap: u16,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub startup_commands: Vec<String>,
    pub keybindings: Vec<KeybindingsConfig>,
    pub window: WindowConfig,
    pub workspaces: WorkspacesConfig,
    pub switch_workspace_on_window_workspace_change: bool,
}

impl Config {
    pub fn new(path: &str) -> Self {
        let config_data = fs::read_to_string(Path::new(path)).unwrap();
        let mut config: Config = toml::from_str(&config_data).unwrap();
        config.window.border.color_inactive_u32 =
            Self::try_color_from_str(&config.window.border.color_inactive).or(Some(2837849));
        config.window.border.color_active_u32 =
            Self::try_color_from_str(&config.window.border.color_active).or(Some(10702339));
        config
    }

    pub fn try_color_from_str(str_color: &str) -> Option<u32> {
        let hex = str_color.trim_start_matches('#');
        if let Ok(res) = u32::from_str_radix(hex, 16) {
            Some(res)
        } else {
            None
        }
    }
}

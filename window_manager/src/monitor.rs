use log::debug;
use xcb::x;

use crate::{
    config::Config,
    window::{Window, WindowType},
    workspace::Workspace,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Rect {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug)]
pub struct Monitor {
    pub root_window: x::Window,
    pub rect: Rect,
    workspaces: Vec<Workspace>,
    focused_workspace_index: usize,
    available_area: Rect,
    docked_windows: Vec<Window>,
}

impl Monitor {
    pub fn new(
        root_window: x::Window,
        rect: Rect,
        config: &Config,
        conn: &xcb::Connection,
    ) -> Self {
        let workspaces_count = if config.workspaces.count > 0 {
            config.workspaces.count
        } else {
            9
        };
        let mut workspaces = Vec::new();
        workspaces.reserve(workspaces_count);
        workspaces.push(Workspace::new(1, rect.clone(), config, conn, false));
        debug!("creating new workspace: {}", 1);

        Self {
            root_window,
            rect: rect.clone(),
            workspaces,
            focused_workspace_index: 0,
            available_area: rect,
            docked_windows: Vec::new(),
        }
    }

    pub fn handle_screen_change(&mut self, width: u16, height: u16) {
        self.rect.width = width;
        self.rect.height = height;
        if self.available_area.width > self.rect.width {
            self.available_area.width = self.rect.width;
        }
        if self.available_area.height > self.rect.height {
            self.available_area.height = self.rect.height;
        }
    }

    pub fn add_tiling_window_to_focused_workspace(
        &mut self,
        conn: &xcb::Connection,
        config: &Config,
        window_id: x::Window,
    ) {
        if let Some(focused_workspace) = self.workspaces.get_mut(self.focused_workspace_index) {
            let expected_width =
                (self.available_area.width as f32 * config.window.default_width_tiling) as u16;
            focused_workspace.add_window_tiling(conn, window_id, expected_width);
        }
    }

    pub fn set_workspace_focused(&mut self, id: u16, config: &Config, conn: &xcb::Connection) {
        debug!(
            "attempt to change focused workspace from {} to {}",
            self.workspaces
                .get(self.focused_workspace_index)
                .unwrap()
                .id,
            id
        );
        if let Some(currently_focused) = self.workspaces.get_mut(self.focused_workspace_index) {
            if currently_focused.id == id {
                return;
            }
            currently_focused.hide_all_windows(self.rect.height, conn);
        }
        if let Some((index, workspace)) = self
            .workspaces
            .iter_mut()
            .enumerate()
            .find(|(_, w)| w.id == id)
        {
            workspace.unhide_all_windows(self.rect.height, conn);
            self.focused_workspace_index = index;
        } else {
            debug!("creating new workspace: {}", id);
            self.workspaces
                .push(Workspace::new(id, self.rect.clone(), config, conn, false));
            self.focused_workspace_index = self.workspaces.len() - 1;
        }
    }

    #[inline]
    pub fn get_focused_workspace(&self) -> Option<&Workspace> {
        let focused_workspace = self.workspaces.get(self.focused_workspace_index);
        if focused_workspace.is_some() {
            focused_workspace
        } else {
            None
        }
    }

    #[inline]
    pub fn get_focused_workspace_mut(&mut self) -> Option<&mut Workspace> {
        let focused_workspace = self.workspaces.get_mut(self.focused_workspace_index);
        if focused_workspace.is_some() {
            focused_workspace
        } else {
            None
        }
    }

    pub fn get_upper_workspace_id(&self) -> Option<u16> {
        if let Some(focused_workspace) = self.workspaces.get(self.focused_workspace_index) {
            let mut upper_id: u16 = 0;
            self.workspaces.iter().for_each(|w| {
                if w.id > focused_workspace.id && upper_id < w.id {
                    upper_id = w.id;
                }
            });
            if upper_id != 0 {
                return Some(upper_id);
            }
        }
        None
    }

    pub fn get_lower_workspace_id(&self) -> Option<u16> {
        if let Some(focused_workspace) = self.workspaces.get(self.focused_workspace_index) {
            let mut lower_id: u16 = u16::MAX;
            self.workspaces.iter().for_each(|w| {
                if w.id < focused_workspace.id && lower_id > w.id {
                    lower_id = w.id;
                }
            });
            if lower_id != u16::MAX {
                return Some(lower_id);
            }
        }
        None
    }
}

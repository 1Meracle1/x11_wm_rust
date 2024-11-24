use xcb::x;

use crate::{config::Config, window::Window, workspace::Workspace};

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
    focused_workspace: usize,
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
        workspaces.push(Workspace::new(rect.clone(), config, conn));

        Self {
            root_window,
            rect: rect.clone(),
            workspaces,
            focused_workspace: 0,
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

    pub fn make_rect_for_tiling_window(&mut self, conn: &xcb::Connection, config: &Config) -> Rect {
        let width_percent = if config.window.default_width_tiling > 0.0 {
            config.window.default_width_tiling
        } else {
            3.33
        };
        let width = (self.available_area.width as f32 * width_percent) as u16;
        if let Some(focused_workspace) = self.workspaces.get_mut(self.focused_workspace) {
            focused_workspace.make_space_for_tiling_window(width, conn)
        } else {
            if self.workspaces.is_empty() {
                self.workspaces
                    .push(Workspace::new(self.available_area.clone(), config, conn));
            }
            self.workspaces
                .first_mut()
                .unwrap()
                .make_space_for_tiling_window(width, conn)
        }
    }

    pub fn set_focused(&mut self, window: &Window, conn: &xcb::Connection, config: &Config) {
        if let Some(focused_workspace) = self.workspaces.get_mut(self.focused_workspace) {
            focused_workspace.set_focused(window, conn, config);
        }
    }
}

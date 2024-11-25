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

    pub fn set_focused(&mut self, window: &Window, conn: &xcb::Connection, via_keyboard: bool) {
        if let Some(focused_workspace) = self.workspaces.get_mut(self.focused_workspace) {
            focused_workspace.set_focused(window, conn, via_keyboard);
        }
    }

    pub fn add_window_tiling(
        &mut self,
        conn: &xcb::Connection,
        config: &Config,
        window_id: x::Window,
    ) {
        if let Some(focused_workspace) = self.workspaces.get_mut(self.focused_workspace) {
            let expected_width =
                (self.available_area.width as f32 * config.window.default_width_tiling) as u16;
            focused_workspace.add_window_tiling(conn, config, window_id, expected_width);
        }
    }

    pub fn get_focused_workspace(&self) -> Option<&Workspace> {
        let focused_workspace = self.workspaces.get(self.focused_workspace);
        if focused_workspace.is_some() {
            focused_workspace
        } else {
            None
        }
    }

    #[inline]
    pub fn get_focused_workspace_mut(&mut self) -> Option<&mut Workspace> {
        let focused_workspace = self.workspaces.get_mut(self.focused_workspace);
        if focused_workspace.is_some() {
            focused_workspace
        } else {
            None
        }
    }

    pub fn get_window(&self, window_xcb: x::Window) -> Option<&Window> {
        if let Some(focused_workspace) = self.workspaces.get(self.focused_workspace) {
            focused_workspace.get_window(window_xcb)
        } else {
            None
        }
    }

    pub fn get_window_mut(&mut self, window_xcb: x::Window) -> Option<&mut Window> {
        if let Some(focused_workspace) = self.workspaces.get_mut(self.focused_workspace) {
            focused_workspace.get_window_mut(window_xcb)
        } else {
            None
        }
    }
}

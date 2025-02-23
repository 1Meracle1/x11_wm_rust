use base::Rect;
use x11_bindings::bindings::xcb_window_t;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Window {
    pub window: xcb_window_t,
    pub rect: Rect,
    pub visible: bool,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct DockedWindows {
    windows: Vec<xcb_window_t>,
    rects: Vec<Rect>,
}

impl DockedWindows {
    #[allow(dead_code)]
    pub fn new(reserved_capacity: usize) -> Self {
        let mut windows = Vec::new();
        let mut rects = Vec::new();
        if reserved_capacity > 0 {
            windows.reserve(reserved_capacity);
            rects.reserve(reserved_capacity);
        }
        Self { windows, rects }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn at(&self, index: usize) -> Option<(xcb_window_t, &Rect)> {
        if index < self.windows.len() {
            Some((self.windows[index], &self.rects[index]))
        } else {
            None
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn at_rect(&self, index: usize) -> Option<&Rect> {
        if index < self.rects.len() {
            Some(&self.rects[index])
        } else {
            None
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn at_window(&self, index: usize) -> Option<xcb_window_t> {
        if index < self.windows.len() {
            Some(self.windows[index])
        } else {
            None
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn change_rect(&mut self, index: usize, new_rect: Rect) {
        assert!(index < self.windows.len());
        self.rects[index] = new_rect;
    }

    #[allow(dead_code)]
    #[inline]
    pub fn add(&mut self, window: xcb_window_t, rect: Rect) {
        self.windows.push(window);
        self.rects.push(rect);
    }

    #[allow(dead_code)]
    #[inline]
    pub fn remove_window(&mut self, window: xcb_window_t) {
        if let Some((idx, _)) = self.windows.iter().enumerate().find(|(_, w)| **w == window) {
            self.windows.remove(idx);
            self.rects.remove(idx);
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn remove(&mut self, index: usize) {
        assert!(index < self.windows.len());
        self.windows.remove(index);
        self.rects.remove(index);
    }

    #[allow(dead_code)]
    #[inline]
    pub fn rects(&mut self) -> &Vec<Rect> {
        &self.rects
    }

    #[allow(dead_code)]
    #[inline]
    pub fn rects_iter_mut(&mut self) -> impl Iterator<Item = &mut Rect> {
        self.rects.iter_mut()
    }

    #[allow(dead_code)]
    #[inline]
    pub fn windows_iter(&self) -> impl Iterator<Item = &xcb_window_t> {
        self.windows.iter()
    }

    #[allow(dead_code)]
    #[inline]
    pub fn index_of(&self, window: xcb_window_t) -> Option<usize> {
        if let Some((idx, _)) = self.windows.iter().enumerate().find(|(_, w)| **w == window) {
            Some(idx)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    #[allow(dead_code)]
    #[inline]
    pub fn len(&self) -> usize {
        self.windows.len()
    }
}

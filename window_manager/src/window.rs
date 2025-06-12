use std::ops::RangeBounds;

use base::Rect;
use x11_bindings::bindings::xcb_window_t;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Window {
    pub window: xcb_window_t,
    pub rect: Rect,
    pub visible: bool,
}

#[derive(Debug)]
pub struct WindowsCollection {
    windows: Vec<xcb_window_t>,
    rects: Vec<Rect>,
    visibles: Vec<bool>,
}

impl WindowsCollection {
    pub fn new(reserved_capacity: usize) -> Self {
        let mut windows = Vec::new();
        let mut rects = Vec::new();
        let mut visibles = Vec::new();
        if reserved_capacity > 0 {
            windows.reserve(reserved_capacity);
            rects.reserve(reserved_capacity);
            visibles.reserve(reserved_capacity);
        }
        Self {
            windows,
            rects,
            visibles,
        }
    }

    pub fn sort_by_rect_x_asc(&mut self) {
        let mut indices: Vec<usize> = (0..self.rects.len()).collect();
        indices.sort_by_key(|&i| self.rects[i].x);
        self.windows = indices.iter().map(|&i| self.windows[i].clone()).collect();
        self.rects = indices.iter().map(|&i| self.rects[i].clone()).collect();
        self.visibles = indices.iter().map(|&i| self.visibles[i].clone()).collect();
    }

    pub fn sort_by_rect_y_asc(&mut self) {
        let mut indices: Vec<usize> = (0..self.rects.len()).collect();
        indices.sort_by_key(|&i| self.rects[i].y);
        self.windows = indices.iter().map(|&i| self.windows[i].clone()).collect();
        self.rects = indices.iter().map(|&i| self.rects[i].clone()).collect();
        self.visibles = indices.iter().map(|&i| self.visibles[i].clone()).collect();
    }

    #[inline]
    pub fn add(&mut self, window: xcb_window_t, rect: Rect, visible: bool) {
        self.windows.push(window);
        self.rects.push(rect);
        self.visibles.push(visible);
    }

    #[inline]
    pub fn remove_at(&mut self, index: usize) -> (xcb_window_t, Rect, bool) {
        (
            self.windows.remove(index),
            self.rects.remove(index),
            self.visibles.remove(index),
        )
    }

    #[inline]
    pub fn at(&self, index: usize) -> Option<(xcb_window_t, &Rect)> {
        if index < self.windows.len() {
            Some((self.windows[index], &self.rects[index]))
        } else {
            None
        }
    }

    #[inline]
    pub fn at_rect(&self, index: usize) -> Option<&Rect> {
        if index < self.rects.len() {
            Some(&self.rects[index])
        } else {
            None
        }
    }

    #[inline]
    pub fn at_rect_mut(&mut self, index: usize) -> Option<&mut Rect> {
        if index < self.rects.len() {
            Some(&mut self.rects[index])
        } else {
            None
        }
    }

    #[inline]
    pub fn at_window(&self, index: usize) -> Option<xcb_window_t> {
        if index < self.windows.len() {
            Some(self.windows[index])
        } else {
            None
        }
    }

    #[inline]
    pub fn at_visible(&self, index: usize) -> Option<bool> {
        if index < self.visibles.len() {
            Some(self.visibles[index])
        } else {
            None
        }
    }

    #[inline]
    pub fn index(&self, index: usize) -> (xcb_window_t, &Rect) {
        (self.windows[index], &self.rects[index])
    }

    #[inline]
    pub fn index_rect(&self, index: usize) -> &Rect {
        &self.rects[index]
    }

    #[inline]
    pub fn index_rect_mut(&mut self, index: usize) -> &mut Rect {
        &mut self.rects[index]
    }

    #[inline]
    pub fn index_window(&self, index: usize) -> xcb_window_t {
        self.windows[index]
    }

    #[inline]
    pub fn update_rect_at(&mut self, index: usize, new_rect: Rect) {
        self.rects[index] = new_rect;
    }

    #[inline]
    pub fn swap_windows(&mut self, index_lhs: usize, index_rhs: usize) {
        self.windows.swap(index_lhs, index_rhs);
    }

    #[inline]
    pub fn swap_visibles(&mut self, index_lhs: usize, index_rhs: usize) {
        self.visibles.swap(index_lhs, index_rhs);
    }

    #[inline]
    pub fn swap_rects(&mut self, index_lhs: usize, index_rhs: usize) {
        self.rects.swap(index_lhs, index_rhs);
    }

    #[inline]
    pub fn iter(&self) -> WindowsCollectionIter<'_> {
        WindowsCollectionIter {
            windows_iter: self.windows.iter(),
            rects_iter: self.rects.iter(),
            visibles_iter: self.visibles.iter(),
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> WindowsCollectionIterMut<'_> {
        WindowsCollectionIterMut {
            windows_iter: self.windows.iter_mut(),
            rects_iter: self.rects.iter_mut(),
            visibles_iter: self.visibles.iter_mut(),
        }
    }

    #[inline]
    pub fn rect_iter(&self) -> std::slice::Iter<'_, Rect> {
        self.rects.iter()
    }

    #[inline]
    pub fn rect_iter_mut(&mut self) -> std::slice::IterMut<'_, Rect> {
        self.rects.iter_mut()
    }

    #[inline]
    pub fn window_iter(&self) -> std::slice::Iter<'_, xcb_window_t> {
        self.windows.iter()
    }

    #[inline]
    pub fn visible_iter(&self) -> std::slice::Iter<'_, bool> {
        self.visibles.iter()
    }

    #[inline]
    pub fn rects(&self) -> &Vec<Rect> {
        &self.rects
    }

    #[inline]
    pub fn rects_slice<R>(&self, range: R) -> &[Rect]
    where
        R: RangeBounds<usize>,
    {
        let start = match range.start_bound() {
            std::ops::Bound::Included(&n) => n,
            std::ops::Bound::Excluded(&n) => n + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&n) => n + 1, // +1 because slice end is exclusive
            std::ops::Bound::Excluded(&n) => n,
            std::ops::Bound::Unbounded => self.rects.len(),
        };
        &self.rects[start..end]
    }

    #[inline]
    pub fn rects_slice_mut<R>(&mut self, range: R) -> &mut [Rect]
    where
        R: RangeBounds<usize>,
    {
        let start = match range.start_bound() {
            std::ops::Bound::Included(&n) => n,
            std::ops::Bound::Excluded(&n) => n + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&n) => n + 1, // +1 because slice end is exclusive
            std::ops::Bound::Excluded(&n) => n,
            std::ops::Bound::Unbounded => self.rects.len(),
        };
        &mut self.rects[start..end]
    }

    #[inline]
    pub fn index_of(&self, window: xcb_window_t) -> Option<usize> {
        if let Some((idx, _)) = self.windows.iter().enumerate().find(|(_, w)| **w == window) {
            Some(idx)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.windows.len()
    }
}

pub struct WindowsCollectionIterMut<'a> {
    windows_iter: std::slice::IterMut<'a, xcb_window_t>,
    rects_iter: std::slice::IterMut<'a, Rect>,
    visibles_iter: std::slice::IterMut<'a, bool>,
}

impl<'a> Iterator for WindowsCollectionIterMut<'a> {
    type Item = (&'a mut xcb_window_t, &'a mut Rect, &'a mut bool);

    fn next(&mut self) -> Option<Self::Item> {
        let window = self.windows_iter.next()?;
        let rect = self.rects_iter.next()?;
        let visible = self.visibles_iter.next()?;
        Some((window, rect, visible))
    }
}

impl<'a> IntoIterator for &'a mut WindowsCollection {
    type Item = (&'a mut xcb_window_t, &'a mut Rect, &'a mut bool);

    type IntoIter = WindowsCollectionIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        WindowsCollectionIterMut {
            windows_iter: self.windows.iter_mut(),
            rects_iter: self.rects.iter_mut(),
            visibles_iter: self.visibles.iter_mut(),
        }
    }
}

pub struct WindowsCollectionIter<'a> {
    windows_iter: std::slice::Iter<'a, xcb_window_t>,
    rects_iter: std::slice::Iter<'a, Rect>,
    visibles_iter: std::slice::Iter<'a, bool>,
}

impl<'a> Iterator for WindowsCollectionIter<'a> {
    type Item = (&'a xcb_window_t, &'a Rect, &'a bool);

    fn next(&mut self) -> Option<Self::Item> {
        let window = self.windows_iter.next()?;
        let rect = self.rects_iter.next()?;
        let visible = self.visibles_iter.next()?;
        Some((window, rect, visible))
    }
}

impl<'a> IntoIterator for &'a WindowsCollection {
    type Item = (&'a xcb_window_t, &'a Rect, &'a bool);

    type IntoIter = WindowsCollectionIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        WindowsCollectionIter {
            windows_iter: self.windows.iter(),
            rects_iter: self.rects.iter(),
            visibles_iter: self.visibles.iter(),
        }
    }
}

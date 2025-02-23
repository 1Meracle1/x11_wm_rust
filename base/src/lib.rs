#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RectSide {
    Top,
    Bottom,
    Left,
    Right,
}

#[allow(dead_code)]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    #[allow(dead_code)]
    #[inline]
    pub fn intersects_with(&self, other: &Rect) -> bool {
        if self.x + self.width as i32 <= other.x {
            return false;
        }
        if self.y + self.height as i32 <= other.y {
            return false;
        }
        if self.x >= other.x + other.width as i32 {
            return false;
        }
        if self.y >= other.y + other.height as i32 {
            return false;
        }
        return true;
    }

    #[allow(dead_code)]
    pub fn clamp_to(&self, parent: &Rect) -> Rect {
        let x = self.x.clamp(parent.x, parent.x + parent.width as i32);
        let y = self.y.clamp(parent.y, parent.y + parent.height as i32);
        let width = self
            .width
            .min((parent.width as i32 - (parent.x - x).abs()) as u32);
        let height = self
            .height
            .min((parent.height as i32 - (parent.y - y).abs()) as u32);
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn add_padding(&self, horizontal: u32, vertical: u32) -> Rect {
        Rect {
            x: self.x + horizontal as i32,
            y: self.y + vertical as i32,
            width: self.width - horizontal * 2,
            height: self.height - vertical * 2,
        }
    }

    /// Returns center x,y position
    #[allow(dead_code)]
    #[inline]
    pub fn center(&self) -> (i32, i32) {
        (
            self.x + self.width as i32 / 2,
            self.y + self.height as i32 / 2,
        )
    }

    #[allow(dead_code)]
    pub fn available_rect_after_adding_rects(&self, child_rects: &Vec<Rect>) -> Rect {
        let mut avail_rect = self.clone();
        for rect in child_rects {
            avail_rect = avail_rect.available_rect_after_adding_rect(&rect)
        }
        avail_rect
    }

    // #[allow(dead_code)]
    // pub fn available_rect_after_adding_rects_inner_gap(
    //     &self,
    //     child_rects: &Vec<Rect>,
    //     inner_gap: u32,
    // ) -> Rect {
    //     let mut avail_rect = self.clone();
    //     for (idx, rect) in child_rects.iter().enumerate() {
    //         avail_rect = avail_rect.available_rect_after_adding_rect(&rect);
    //         if idx > 0 {
    //             avail_rect.x += inner_gap as i32;
    //         }
    //     }
    //     avail_rect
    // }

    #[allow(dead_code)]
    pub fn available_rect_after_adding_rect(&self, added_rect: &Rect) -> Rect {
        let (center_x, center_y) = self.center();
        let new_rect = self.new_rect_magnified(added_rect);
        if new_rect.width > new_rect.height {
            if new_rect.y < center_y {
                let x = self.x;
                let y = new_rect.y + new_rect.height as i32;
                let width = self.width;
                let height = ((self.y + self.height as i32) - y) as u32;
                Rect {
                    x,
                    y,
                    width,
                    height,
                }
            } else {
                let x = self.x;
                let y = self.y;
                let width = self.width;
                let height = (new_rect.y - self.y) as u32;
                Rect {
                    x,
                    y,
                    width,
                    height,
                }
            }
        } else {
            if new_rect.x < center_x {
                let x = new_rect.x + new_rect.width as i32;
                let y = self.y;
                let width = ((self.x + self.width as i32) - x) as u32;
                let height = self.height;
                Rect {
                    x,
                    y,
                    width,
                    height,
                }
            } else {
                let x = self.x;
                let y = self.y;
                let width = (new_rect.x - self.x) as u32;
                let height = self.height;
                Rect {
                    x,
                    y,
                    width,
                    height,
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn new_rect_magnified_padding(
        &self,
        new_rect: &Rect,
        padding_x: u32,
        padding_y: u32,
    ) -> Rect {
        let (center_x, center_y) = self.center();
        if new_rect.width > new_rect.height {
            if new_rect.y < center_y {
                let x = new_rect.x.clamp(
                    self.x + padding_x as i32,
                    self.x + self.width as i32 - (padding_x * 2) as i32,
                );
                let y = self.y + padding_y as i32;
                let width = new_rect
                    .width
                    .clamp(0, (self.width as i32 - (self.x - x).abs()) as u32);
                let height = new_rect
                    .height
                    .clamp(0, (self.height as i32 - (self.y - y).abs()) as u32);

                Rect {
                    x,
                    y,
                    width,
                    height,
                }
            } else {
                let x = new_rect.x.clamp(self.x, self.x + self.width as i32);
                let height = new_rect.height.clamp(0, self.height);
                let y = self.y + self.height as i32 - height as i32;
                let width = new_rect
                    .width
                    .clamp(0, (self.width as i32 - (self.x - x).abs()) as u32);

                Rect {
                    x,
                    y,
                    width,
                    height,
                }
            }
        } else {
            if new_rect.x < center_x {
                let x = self.x;
                let y = new_rect.y.clamp(self.y, self.y + self.height as i32);
                let width = new_rect
                    .width
                    .clamp(0, (self.width as i32 - (self.x - x).abs()) as u32);
                let height = new_rect
                    .height
                    .clamp(0, (self.height as i32 - (self.y - y).abs()) as u32);

                Rect {
                    x,
                    y,
                    width,
                    height,
                }
            } else {
                let width = new_rect.width.clamp(0, self.width);
                let x = self.x + self.width as i32 - width as i32;
                let y = new_rect.y.clamp(self.y, self.y + self.height as i32);
                let height = new_rect
                    .height
                    .clamp(0, (self.height as i32 - (self.y - y).abs()) as u32);

                Rect {
                    x,
                    y,
                    width,
                    height,
                }
            }
        }
    }

    /// Returns rectangle magnified to either of the sides of the parent rectangle
    #[allow(dead_code)]
    pub fn new_rect_magnified(&self, new_rect: &Rect) -> Rect {
        let (center_x, center_y) = self.center();
        if new_rect.width > new_rect.height {
            if new_rect.y < center_y {
                let x = new_rect.x.clamp(self.x, self.x + self.width as i32);
                let y = self.y;
                let width = new_rect
                    .width
                    .clamp(0, (self.width as i32 - (self.x - x).abs()) as u32);
                let height = new_rect
                    .height
                    .clamp(0, (self.height as i32 - (self.y - y).abs()) as u32);

                Rect {
                    x,
                    y,
                    width,
                    height,
                }
            } else {
                let x = new_rect.x.clamp(self.x, self.x + self.width as i32);
                let height = new_rect.height.clamp(0, self.height);
                let y = self.y + self.height as i32 - height as i32;
                let width = new_rect
                    .width
                    .clamp(0, (self.width as i32 - (self.x - x).abs()) as u32);

                Rect {
                    x,
                    y,
                    width,
                    height,
                }
            }
        } else {
            if new_rect.x < center_x {
                let x = self.x;
                let y = new_rect.y.clamp(self.y, self.y + self.height as i32);
                let width = new_rect
                    .width
                    .clamp(0, (self.width as i32 - (self.x - x).abs()) as u32);
                let height = new_rect
                    .height
                    .clamp(0, (self.height as i32 - (self.y - y).abs()) as u32);

                Rect {
                    x,
                    y,
                    width,
                    height,
                }
            } else {
                let width = new_rect.width.clamp(0, self.width);
                let x = self.x + self.width as i32 - width as i32;
                let y = new_rect.y.clamp(self.y, self.y + self.height as i32);
                let height = new_rect
                    .height
                    .clamp(0, (self.height as i32 - (self.y - y).abs()) as u32);

                Rect {
                    x,
                    y,
                    width,
                    height,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersects_positive() {
        let lhs = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 100,
        };
        let rhs = Rect {
            x: 20,
            y: 20,
            width: 100,
            height: 100,
        };
        let result = lhs.intersects_with(&rhs);
        assert!(result);
    }

    #[test]
    fn intersects_negative() {
        let lhs = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 100,
        };
        let rhs = Rect {
            x: 101,
            y: 20,
            width: 100,
            height: 100,
        };
        let result = lhs.intersects_with(&rhs);
        assert!(!result);
    }

    #[test]
    fn center() {
        let rect = Rect {
            x: 2,
            y: 2,
            width: 10,
            height: 8,
        };
        let expected_center: (i32, i32) = (7, 6);
        let actual_center = rect.center();
        assert_eq!(expected_center, actual_center);
    }

    #[test]
    fn new_rect_magnified_top() {
        let parent_rect = Rect {
            x: 2,
            y: 2,
            width: 10,
            height: 8,
        };
        let added_rect = Rect {
            x: 0,
            y: 0,
            width: 4,
            height: 3,
        };
        let expected_magnified_rect = Rect {
            x: 2,
            y: 2,
            width: 4,
            height: 3,
        };
        let actual_magnified_rect = parent_rect.new_rect_magnified(&added_rect);
        assert_eq!(expected_magnified_rect, actual_magnified_rect);
    }

    #[test]
    fn new_rect_magnified_bottom() {
        let parent_rect = Rect {
            x: 2,
            y: 2,
            width: 10,
            height: 8,
        };
        let added_rect = Rect {
            x: 0,
            y: 8,
            width: 4,
            height: 3,
        };
        let expected_magnified_rect = Rect {
            x: 2,
            y: 7,
            width: 4,
            height: 3,
        };
        let actual_magnified_rect = parent_rect.new_rect_magnified(&added_rect);
        assert_eq!(expected_magnified_rect, actual_magnified_rect);
    }

    #[test]
    fn new_rect_magnified_left() {
        let parent_rect = Rect {
            x: 2,
            y: 2,
            width: 10,
            height: 8,
        };
        let added_rect = Rect {
            x: 4,
            y: 0,
            width: 3,
            height: 4,
        };
        let expected_magnified_rect = Rect {
            x: 2,
            y: 2,
            width: 3,
            height: 4,
        };
        let actual_magnified_rect = parent_rect.new_rect_magnified(&added_rect);
        assert_eq!(expected_magnified_rect, actual_magnified_rect);
    }

    #[test]
    fn new_rect_magnified_right() {
        let parent_rect = Rect {
            x: 2,
            y: 2,
            width: 10,
            height: 8,
        };
        let added_rect = Rect {
            x: 8,
            y: 0,
            width: 3,
            height: 4,
        };
        let expected_magnified_rect = Rect {
            x: 9,
            y: 2,
            width: 3,
            height: 4,
        };
        let actual_magnified_rect = parent_rect.new_rect_magnified(&added_rect);
        assert_eq!(expected_magnified_rect, actual_magnified_rect);
    }

    #[test]
    fn new_rect_magnified_zero_width_added() {
        let parent_rect = Rect {
            x: 2,
            y: 2,
            width: 10,
            height: 8,
        };
        let added_rect = Rect {
            x: 8,
            y: 0,
            width: 0,
            height: 4,
        };
        let expected_magnified_rect = Rect {
            x: 12,
            y: 2,
            width: 0,
            height: 4,
        };
        let actual_magnified_rect = parent_rect.new_rect_magnified(&added_rect);
        assert_eq!(expected_magnified_rect, actual_magnified_rect);
    }

    #[test]
    fn new_rect_magnified_zero_height_added() {
        let parent_rect = Rect {
            x: 2,
            y: 2,
            width: 10,
            height: 8,
        };
        let added_rect = Rect {
            x: 4,
            y: 0,
            width: 3,
            height: 0,
        };
        let expected_magnified_rect = Rect {
            x: 4,
            y: 2,
            width: 3,
            height: 0,
        };
        let actual_magnified_rect = parent_rect.new_rect_magnified(&added_rect);
        assert_eq!(expected_magnified_rect, actual_magnified_rect);
    }

    #[test]
    fn new_rect_magnified_zero_width_parent_top() {
        let parent_rect = Rect {
            x: 2,
            y: 2,
            width: 0,
            height: 8,
        };
        let added_rect = Rect {
            x: 4,
            y: 0,
            width: 3,
            height: 4,
        };
        let expected_magnified_rect = Rect {
            x: 2,
            y: 2,
            width: 0,
            height: 4,
        };
        let actual_magnified_rect = parent_rect.new_rect_magnified(&added_rect);
        assert_eq!(expected_magnified_rect, actual_magnified_rect);
    }

    #[test]
    fn new_rect_magnified_zero_width_parent_left() {
        let parent_rect = Rect {
            x: 2,
            y: 2,
            width: 0,
            height: 8,
        };
        let added_rect = Rect {
            x: 3,
            y: 0,
            width: 4,
            height: 3,
        };
        let expected_magnified_rect = Rect {
            x: 2,
            y: 2,
            width: 0,
            height: 3,
        };
        let actual_magnified_rect = parent_rect.new_rect_magnified(&added_rect);
        assert_eq!(expected_magnified_rect, actual_magnified_rect);
    }

    #[test]
    fn available_rect_after_adding_rect_top() {
        let parent_rect = Rect {
            x: 2,
            y: 2,
            width: 10,
            height: 8,
        };
        let added_rect = Rect {
            x: 0,
            y: 0,
            width: 4,
            height: 3,
        };
        let expected_avail_rect = Rect {
            x: 2,
            y: 5,
            width: 10,
            height: 5,
        };
        let actual_avail_rect = parent_rect.available_rect_after_adding_rect(&added_rect);
        assert_eq!(expected_avail_rect, actual_avail_rect);
    }

    #[test]
    fn available_rect_after_adding_rect_bottom() {
        let parent_rect = Rect {
            x: 2,
            y: 2,
            width: 10,
            height: 8,
        };
        let added_rect = Rect {
            x: 0,
            y: 8,
            width: 4,
            height: 3,
        };
        let expected_avail_rect = Rect {
            x: 2,
            y: 2,
            width: 10,
            height: 5,
        };
        let actual_avail_rect = parent_rect.available_rect_after_adding_rect(&added_rect);
        assert_eq!(expected_avail_rect, actual_avail_rect);
    }

    #[test]
    fn available_rect_after_adding_rect_left() {
        let parent_rect = Rect {
            x: 2,
            y: 2,
            width: 10,
            height: 8,
        };
        let added_rect = Rect {
            x: 4,
            y: 0,
            width: 3,
            height: 4,
        };
        let expected_avail_rect = Rect {
            x: 5,
            y: 2,
            width: 7,
            height: 8,
        };
        let actual_avail_rect = parent_rect.available_rect_after_adding_rect(&added_rect);
        assert_eq!(expected_avail_rect, actual_avail_rect);
    }

    #[test]
    fn available_rect_after_adding_rect_right() {
        let parent_rect = Rect {
            x: 2,
            y: 2,
            width: 10,
            height: 8,
        };
        let added_rect = Rect {
            x: 8,
            y: 0,
            width: 3,
            height: 4,
        };
        let expected_avail_rect = Rect {
            x: 2,
            y: 2,
            width: 7,
            height: 8,
        };
        let actual_avail_rect = parent_rect.available_rect_after_adding_rect(&added_rect);
        assert_eq!(expected_avail_rect, actual_avail_rect);
    }
}

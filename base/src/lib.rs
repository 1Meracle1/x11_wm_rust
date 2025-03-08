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
    pub fn clamp_to_y(&self, parent: &Rect) -> Rect {
        let x = self.x;
        let y = self.y.clamp(parent.y, parent.y + parent.height as i32);
        let width = self.width;
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

    #[inline]
    pub fn distance_between_centers(&self, other: &Rect) -> i32 {
        let (x1, y1) = self.center();
        let (x2, y2) = other.center();
        ((x2 - x1).pow(2) + (y2 - y1).pow(2)).isqrt()
    }

    #[allow(dead_code)]
    pub fn available_rect_after_adding_rects(
        &self,
        child_rects: std::slice::Iter<'_, Rect>,
    ) -> Rect {
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
    //     avail_rec
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

    /// self is a parent rect / available rect
    ///
    /// calculates available space within the parent rectangle to the left of the first rect
    /// and to the right of the last rect
    #[allow(dead_code)]
    pub fn available_space(&self, first_rect: &Rect, last_rect: &Rect) -> (i32, i32) {
        let free_space_left_side = first_rect.x - self.x;
        let free_space_right_side =
            self.x + self.width as i32 - last_rect.x - last_rect.width as i32;
        (free_space_left_side, free_space_right_side)
    }

    /// self is a parent rect / available rect
    ///
    /// assumes that `rects` is sorted
    #[allow(dead_code)]
    pub fn first_visible_rect_idx_horiz(&self, rects: &[Rect]) -> Option<usize> {
        if rects.is_empty() {
            None
        } else {
            if let Some((index, _)) = rects
                .iter()
                .enumerate()
                .find(|(_, r)| r.x >= self.x && r.x <= self.x + self.width as i32)
            {
                Some(index)
            } else {
                None
            }
        }
    }

    /// self is a parent rect / available rect
    ///
    /// assumes that `rects` is sorted
    #[allow(dead_code)]
    pub fn last_visible_rect_idx_horiz(&self, rects: &[Rect]) -> Option<usize> {
        if rects.is_empty() {
            None
        } else {
            if let Some((index, _)) = rects
                .iter()
                .enumerate()
                .rev()
                .find(|(_, r)| r.x >= self.x && r.x <= self.x + self.width as i32)
            {
                Some(index)
            } else {
                None
            }
        }
    }

    /// returns new rectangle and amount of pixels to move to the left and to the right of the focused rect
    ///
    /// self is a parent rect / available rect
    #[allow(dead_code)]
    pub fn calc_new_rect_added_after_focused(
        &self,
        default_width: u32,
        default_height: u32,
        focused_idx: Option<usize>,
        existing_rects: &[Rect],
        inner_gap: u32,
    ) -> (Rect, i32, i32) {
        if existing_rects.is_empty() {
            let new_rect = Rect {
                x: self.x + (self.width as i32 - default_width as i32) / 2, // + config.border_size as i32 * 2,
                y: self.y,
                width: default_width,
                height: default_height,
            };
            let move_lhs = 0;
            let move_rhs = 0;

            (new_rect, move_lhs, move_rhs)
        } else {
            let first_visible_idx = self.first_visible_rect_idx_horiz(existing_rects).unwrap();
            let last_visible_idx = self.last_visible_rect_idx_horiz(existing_rects).unwrap();
            assert!(first_visible_idx <= last_visible_idx);

            let add_after_idx = if let Some(focused_idx) = focused_idx {
                assert!(focused_idx < existing_rects.len());
                focused_idx
            } else {
                last_visible_idx
            };

            let mut new_rect = Rect {
                x: existing_rects[add_after_idx].x
                    + existing_rects[add_after_idx].width as i32
                    + inner_gap as i32,
                y: self.y,
                width: default_width,
                height: default_height,
            };

            let mut move_left_rects_by = 0;
            let mut move_right_rects_by = if add_after_idx == last_visible_idx {
                0
            } else {
                new_rect.width as i32 + inner_gap as i32
            };

            if new_rect.x + new_rect.width as i32 > self.x + self.width as i32 {
                let diff = new_rect.x + new_rect.width as i32 - (self.x + self.width as i32);
                new_rect.x -= diff;
                move_left_rects_by -= diff;
                if add_after_idx != last_visible_idx {
                    move_right_rects_by -= diff;
                }
            }

            let last_rect = if add_after_idx == last_visible_idx {
                new_rect.clone()
            } else {
                let mut last_rect = existing_rects[last_visible_idx].clone();
                last_rect.x += move_right_rects_by;
                last_rect
            };
            let first_rect = if move_left_rects_by != 0 {
                let mut r = existing_rects[first_visible_idx].clone();
                r.x += move_left_rects_by;
                r
            } else {
                existing_rects[first_visible_idx].clone()
            };
            let (free_space_left_side, free_space_right_side) =
                self.available_space(&first_rect, &last_rect);
            let move_free_space_balance = if free_space_right_side + free_space_left_side >= 0 {
                (free_space_right_side - free_space_left_side) / 2
            } else if free_space_left_side >= 0 {
                -free_space_left_side
            } else if free_space_right_side >= 0 {
                free_space_right_side
            } else {
                0
            };

            new_rect.x += move_free_space_balance;
            move_left_rects_by += move_free_space_balance;
            if add_after_idx != last_visible_idx {
                move_right_rects_by += move_free_space_balance;
            }

            (new_rect, move_left_rects_by, move_right_rects_by)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_space_test() {
        let avail_rect = Rect {
            x: 10,
            y: 10,
            width: 1900,
            height: 1029,
        };
        let first_rect = Rect {
            x: 654,
            y: 10,
            width: 612,
            height: 1029,
        };
        let last_rect = Rect {
            x: 1277,
            y: 10,
            width: 612,
            height: 1029,
        };
        let (actual_free_space_left, actual_free_space_right) =
            avail_rect.available_space(&first_rect, &last_rect);
        assert_eq!(actual_free_space_left, 644);
        assert_eq!(actual_free_space_right, 21);
    }

    #[test]
    fn calc_new_rect_added_after_focused_first_window() {
        let outer_gap_horiz: u32 = 10;
        let outer_gap_vert: u32 = 10;
        let inner_gap: u32 = 5;

        let avail_rect = Rect {
            x: 0 + outer_gap_horiz as i32,
            y: 0 + outer_gap_vert as i32,
            width: 1920 - outer_gap_horiz * 2,
            height: 1080 - outer_gap_vert * 2,
        };

        let default_width = (avail_rect.width as f32 * 0.333) as u32;
        let default_height = avail_rect.height;
        let existing_rects = Vec::new();

        let expected_rect = Rect {
            x: avail_rect.x + (avail_rect.width as i32 - default_width as i32) / 2,
            y: avail_rect.y,
            width: default_width,
            height: default_height,
        };

        let (new_rect, move_left_rects_by, move_right_rects_by) = avail_rect
            .calc_new_rect_added_after_focused(
                default_width,
                default_height,
                None,
                &existing_rects,
                inner_gap,
            );
        assert_eq!(expected_rect, new_rect);
        assert_eq!(move_left_rects_by, 0);
        assert_eq!(move_right_rects_by, 0);
    }

    #[test]
    fn calc_new_rect_added_after_focused_second_window() {
        let avail_rect = Rect {
            x: 10,
            y: 10,
            width: 1900,
            height: 1029,
        };
        let existing_rects = vec![Rect {
            x: 654,
            y: 10,
            width: 612,
            height: 1029,
        }];

        let expected_new_rect = Rect {
            x: 966,
            y: 10,
            width: 612,
            height: 1029,
        };

        let (actual_new_rect, move_left_rects_by, move_right_rects_by) =
            avail_rect.calc_new_rect_added_after_focused(612, 1029, Some(0), &existing_rects, 11);
        assert_eq!(expected_new_rect, actual_new_rect);
        assert_eq!(move_left_rects_by, -311);
        assert_eq!(move_right_rects_by, 0);
    }

    #[test]
    fn calc_new_rect_added_after_focused_third_window() {
        let avail_rect = Rect {
            x: 10,
            y: 10,
            width: 1900,
            height: 1029,
        };
        let existing_rects = vec![
            Rect {
                x: 343,
                y: 10,
                width: 612,
                height: 1029,
            },
            Rect {
                x: 966,
                y: 10,
                width: 612,
                height: 1029,
            },
        ];

        let expected_new_rect = Rect {
            x: 1277,
            y: 10,
            width: 612,
            height: 1029,
        };

        let (actual_new_rect, move_left_rects_by, move_right_rects_by) =
            avail_rect.calc_new_rect_added_after_focused(612, 1029, Some(1), &existing_rects, 11);
        assert_eq!(expected_new_rect, actual_new_rect);
        assert_eq!(move_left_rects_by, -312);
        assert_eq!(move_right_rects_by, 0);
    }

    #[test]
    fn calc_new_rect_added_after_focused_not_last_third_window() {
        let avail_rect = Rect {
            x: 10,
            y: 10,
            width: 1900,
            height: 1029,
        };
        let existing_rects = vec![
            Rect {
                x: 343,
                y: 10,
                width: 612,
                height: 1029,
            },
            Rect {
                x: 966,
                y: 10,
                width: 612,
                height: 1029,
            },
        ];

        let expected_new_rect = Rect {
            x: 654,
            y: 10,
            width: 612,
            height: 1029,
        };

        let (actual_new_rect, move_left_rects_by, move_right_rects_by) =
            avail_rect.calc_new_rect_added_after_focused(612, 1029, Some(0), &existing_rects, 11);
        assert_eq!(expected_new_rect, actual_new_rect);
        assert_eq!(move_left_rects_by, -312);
        assert_eq!(move_right_rects_by, 311);
    }

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

#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
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

    // pub fn add_new_rect_to(&self, existing_rects: &Vec<Rect>, new_rect: &Rect) ->  {}
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
}

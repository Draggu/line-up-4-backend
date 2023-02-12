#[derive(Clone, Copy)]
pub enum Vertical {
    Up,
    Down,
    Straight,
}

#[derive(Clone, Copy)]
pub enum Horizontal {
    Left,
    Right,
}
#[derive(Clone, Copy)]
pub struct IndexRange {
    size: u16,
    is_cyclic: bool,
    cursor: u16,
}

impl IndexRange {
    pub fn new(size: u16, is_cyclic: bool, cursor: u16) -> Self {
        Self {
            size,
            is_cyclic,
            cursor,
        }
    }

    pub fn move_cursor(&mut self, distance: u16, h: Horizontal) -> bool {
        let distance = distance % self.size;

        self.cursor = if distance
            > match h {
                Horizontal::Left => self.cursor,
                Horizontal::Right => self.size - self.cursor,
            } {
            if self.is_cyclic {
                match h {
                    Horizontal::Left => self.cursor + self.size - distance,
                    Horizontal::Right => self.cursor - self.size + distance,
                }
            } else {
                return false;
            }
        } else {
            match h {
                Horizontal::Left => self.cursor - distance,
                Horizontal::Right => self.cursor + distance,
            }
        };

        true
    }

    pub fn as_usize(&self) -> usize {
        self.cursor.into()
    }
}

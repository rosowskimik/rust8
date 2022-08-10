use bitvec::{array::BitArray, order::Msb0, view::BitView, BitArr};

pub type DisplayIter<'a> = bitvec::slice::Iter<'a, u8, Msb0>;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;

#[derive(Debug, Clone, Copy, Default)]
pub struct ChipDisplay(pub BitArr!(for DISPLAY_WIDTH * DISPLAY_HEIGHT, in u8, Msb0));

impl ChipDisplay {
    pub fn new() -> Self {
        Self(BitArray::ZERO)
    }

    pub fn clear(&mut self) {
        self.0 = BitArray::ZERO;
    }

    pub fn draw_sprite(&mut self, (x, y): (u8, u8), sprite: &[u8]) -> bool {
        let (x, y) = (x as usize, y as usize);
        let coord_to_index = |(x, y): (usize, usize)| -> usize {
            (y % DISPLAY_HEIGHT) * DISPLAY_WIDTH + (x % DISPLAY_WIDTH)
        };

        let mut collision = false;

        for (y_off, row) in sprite.iter().enumerate() {
            for (x_off, pixel) in row.view_bits::<Msb0>().iter().enumerate() {
                let index = coord_to_index((x + x_off, y + y_off));
                let mut curr = self.0.get_mut(index).unwrap();

                *curr ^= *pixel;

                if !curr && *pixel {
                    collision = true;
                }
            }
        }

        collision
    }

    pub fn iter(&self) -> DisplayIter {
        self.0.iter()
    }
}

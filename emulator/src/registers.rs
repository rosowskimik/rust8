use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Default)]
pub struct ChipRegisters(pub [u8; 16]);

impl ChipRegisters {
    pub fn new() -> Self {
        Self([0; 16])
    }

    pub fn clear(&mut self) {
        self.0.fill(0);
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.0.as_mut_slice()
    }
}

impl Index<u8> for ChipRegisters {
    type Output = u8;

    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u8> for ChipRegisters {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

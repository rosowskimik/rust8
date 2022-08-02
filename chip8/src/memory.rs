use std::{
    io::Read,
    ops::{Deref, DerefMut},
};

use crate::{Chip8Error, Result};

pub const MEMORY_SIZE: usize = 4096;
pub const BUILTIN_SPRITES: [u8; 80] = [
    0xf0, 0x90, 0x90, 0x90, 0xf0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xf0, 0x10, 0xf0, 0x80, 0xf0, // 2
    0xf0, 0x10, 0xf0, 0x10, 0xf0, // 3
    0x90, 0x90, 0xf0, 0x10, 0x10, // 4
    0xf0, 0x80, 0xf0, 0x10, 0xf0, // 5
    0xf0, 0x80, 0xf0, 0x90, 0xf0, // 6
    0xf0, 0x10, 0x20, 0x40, 0x40, // 7
    0xf0, 0x90, 0xf0, 0x90, 0xf0, // 8
    0xf0, 0x90, 0xf0, 0x10, 0xf0, // 9
    0xf0, 0x90, 0xf0, 0x90, 0x90, // A
    0xe0, 0x90, 0xe0, 0x90, 0xe0, // B
    0xf0, 0x80, 0x80, 0x80, 0xf0, // C
    0xe0, 0x90, 0x90, 0x90, 0xe0, // D
    0xf0, 0x80, 0xf0, 0x80, 0xf0, // E
    0xf0, 0x80, 0xf0, 0x80, 0x80, // F
];

pub(crate) const PROGRAM_START: usize = 0x200;
pub(crate) const MAX_PROGRAM_SIZE: usize = 0xFFF - PROGRAM_START;

#[derive(Debug, Clone)]
pub struct ChipMemory([u8; MEMORY_SIZE]);

impl Deref for ChipMemory {
    type Target = [u8; 4096];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChipMemory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for ChipMemory {
    fn default() -> Self {
        Self::init()
    }
}

impl ChipMemory {
    pub fn init() -> Self {
        let mut mem = [0; MEMORY_SIZE];

        mem[..BUILTIN_SPRITES.len()].copy_from_slice(&BUILTIN_SPRITES);

        Self(mem)
    }

    pub fn init_with<R: Read>(r: R) -> Result<Self> {
        let mut mem = Self::init();

        mem.load_rom(r)?;

        Ok(mem)
    }

    pub fn load_rom<R: Read>(&mut self, mut r: R) -> Result<()> {
        let mut buf = Vec::new();
        r.read_to_end(&mut buf)?;

        if buf.len() > MAX_PROGRAM_SIZE {
            return Err(Chip8Error::InvalidProgramSize(buf.len()));
        }

        self.prog_space_mut()[..buf.len()].copy_from_slice(&buf);

        Ok(())
    }

    pub fn clear(&mut self) {
        self.prog_space_mut().fill(0);
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn prog_space(&self) -> &[u8] {
        &self[PROGRAM_START..]
    }

    #[inline]
    pub(crate) fn prog_space_mut(&mut self) -> &mut [u8] {
        &mut self[PROGRAM_START..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ROM: &[u8] = &[0x01, 0x02, 0x03, 0x04];
    const SIZE_TEST_ROM: &[u8] = &[0x01; MAX_PROGRAM_SIZE + 1];

    #[test]
    fn memory_size() {
        let mem = ChipMemory::init();

        assert_eq!(mem.len(), MEMORY_SIZE);
    }

    #[test]
    fn default_sprites() {
        let mem = ChipMemory::init();

        assert!(mem.starts_with(&BUILTIN_SPRITES));
        assert!(mem.prog_space().iter().all(|&x| x == 0));
    }

    #[test]
    fn load_rom() {
        let mut mem = ChipMemory::init();

        mem.load_rom(TEST_ROM).unwrap();

        assert!(mem.prog_space()[..TEST_ROM.len()]
            .iter()
            .zip(TEST_ROM)
            .all(|(a, b)| a == b));

        assert!(mem.prog_space()[TEST_ROM.len()..].iter().all(|&x| x == 0));
    }

    #[test]
    fn check_rom_size() {
        let mut mem = ChipMemory::init();

        let result = mem.load_rom(SIZE_TEST_ROM);

        assert!(result.is_err());

        if let Chip8Error::InvalidProgramSize(size) = result.unwrap_err() {
            assert_eq!(size, SIZE_TEST_ROM.len());
        } else {
            panic!("Expected InvalidProgramSize");
        }
    }

    #[test]
    fn clear() {
        let mut mem = ChipMemory::init();
        mem.load_rom(TEST_ROM).unwrap();

        mem.clear();

        assert!(mem.prog_space().iter().all(|&x| x == 0));
        assert!(mem.starts_with(&BUILTIN_SPRITES));
    }
}

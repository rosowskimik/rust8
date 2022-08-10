use std::{
    io::{self, Read},
    ops::{Deref, DerefMut},
};

pub const PROGRAM_SPACE_START: usize = 0x200;
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChipOpcode(u16);

impl ChipOpcode {
    pub const fn nnn(&self) -> u16 {
        self.0 & 0x0FFF
    }
    pub const fn n(&self) -> u8 {
        (self.0 & 0x000F) as u8
    }
    pub const fn x(&self) -> u8 {
        ((self.0 & 0x0F00) >> 8) as u8
    }
    pub const fn y(&self) -> u8 {
        ((self.0 & 0x00F0) >> 4) as u8
    }
    pub const fn kk(&self) -> u8 {
        (self.0 & 0x00FF) as u8
    }
    pub const fn upper(&self) -> u8 {
        (self.0 >> 8) as u8
    }
    pub const fn lower(&self) -> u8 {
        self.0 as u8
    }
}

impl Deref for ChipOpcode {
    type Target = u16;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct ChipMemory([u8; MEMORY_SIZE]);

impl ChipMemory {
    pub fn init() -> Self {
        let mut mem = [0; MEMORY_SIZE];

        mem[..BUILTIN_SPRITES.len()].copy_from_slice(&BUILTIN_SPRITES);

        Self(mem)
    }

    pub fn init_with<R: Read>(r: R) -> io::Result<Self> {
        let mut mem = Self::init();

        mem.load_rom(r)?;

        Ok(mem)
    }

    pub fn load_rom<R: Read>(&mut self, r: R) -> io::Result<()> {
        for (dst, src) in self.mut_prog_space().iter_mut().zip(r.bytes()) {
            *dst = src?;
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        self.mut_prog_space().fill(0);
    }

    pub fn fetch_opcode(&self, pc: u16) -> ChipOpcode {
        let first = self[pc as usize];
        let second = self[pc as usize + 1];

        ChipOpcode(u16::from_be_bytes([first, second]))
    }

    pub fn prog_space(&self) -> &[u8] {
        &self[PROGRAM_SPACE_START..]
    }

    pub fn mut_prog_space(&mut self) -> &mut [u8] {
        &mut self[PROGRAM_SPACE_START..]
    }
}

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

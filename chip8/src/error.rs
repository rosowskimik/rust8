use thiserror::Error;

use crate::memory::MAX_PROGRAM_SIZE;

#[derive(Error, Debug)]
pub enum Chip8Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid program size {0} (max {} bytes)", MAX_PROGRAM_SIZE)]
    InvalidProgramSize(usize),
    #[error("Invalid opcode: {0:X}")]
    InvalidOpcode(u16),
    #[error("Invalid memory address: {0:X}")]
    InvalidMemoryAddress(u16),
    #[error("Invalid register: V{0:X}")]
    InvalidRegister(u8),
}

pub type Result<T> = std::result::Result<T, Chip8Error>;

use std::io::{self, Read};

use fastrand::Rng;

use crate::{
    display::{ChipDisplay, DisplayIter},
    keypad::ChipKey,
    memory::{ChipMemory, PROGRAM_SPACE_START},
    registers::ChipRegisters,
    timers::ChipTimers,
};

#[derive(Debug, Clone, Copy)]
pub struct ChipConfig {
    pub modified_shift: bool,
    pub modified_load: bool,
}

impl Default for ChipConfig {
    fn default() -> Self {
        Self {
            modified_shift: true,
            modified_load: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChipEmulator {
    memory: ChipMemory,
    display: ChipDisplay,
    timers: ChipTimers,
    vx: ChipRegisters,
    i: u16,
    pc: u16,
    sp: u8,
    stack: [u16; 16],

    pressed: Option<ChipKey>,
    config: ChipConfig,
    rng: Rng,
}

impl ChipEmulator {
    pub fn init() -> Self {
        Self {
            memory: ChipMemory::init(),
            display: ChipDisplay::new(),
            timers: ChipTimers::new(),
            vx: ChipRegisters::new(),
            i: 0,
            pc: PROGRAM_SPACE_START as u16,
            sp: 0,
            stack: [0; 16],
            pressed: None,
            config: ChipConfig::default(),
            rng: Rng::new(),
        }
    }

    pub fn with_config(config: ChipConfig) -> Self {
        let mut emu = Self::init();
        emu.set_config(config);
        emu
    }

    pub fn set_config(&mut self, config: ChipConfig) {
        self.config = config;
    }

    pub fn load_rom<R: Read>(&mut self, r: R) -> io::Result<()> {
        self.memory.load_rom(r)
    }

    pub fn set_key(&mut self, key: ChipKey) {
        self.pressed = Some(key);
    }

    pub fn reset(&mut self) {
        self.memory.clear();
        self.display.clear();
        self.timers.reset();
        self.vx.clear();
        self.i = 0;
        self.pc = PROGRAM_SPACE_START as u16;
        self.sp = 0;
        self.stack.fill(0);

        self.pressed = None;
    }

    pub fn display(&self) -> DisplayIter {
        self.display.iter()
    }

    pub fn tick(&mut self) {
        self.timers.tick();

        let opcode = self.memory.fetch_opcode(self.pc);
        let mut next_pc = self.pc + 2;

        match opcode.upper() & 0xF0 {
            0x00 => match opcode.lower() {
                // 00E0 - CLS: Clear the display
                0xE0 => self.display.clear(),
                //00EE - RET: Return from a subroutine
                0xEE => {
                    self.sp -= 1;
                    next_pc = self.stack[self.sp as usize];
                }
                // 0nnn - SYS addr: Jump to a machine code routine at nnn
                // Ignored
                _ => (),
            },

            // 1nnn - JP addr: Jump to location nnn
            0x10 => {
                next_pc = opcode.nnn();
            }
            // 2nnn - CALL addr: Call subroutine at nnn
            0x20 => {
                self.stack[self.sp as usize] = next_pc;
                self.sp += 1;
                next_pc = opcode.nnn();
            }
            // 3xkk - SE Vx, byte: Skip next instruction if Vx = kk
            0x30 => {
                if self.vx[opcode.x()] == opcode.kk() {
                    next_pc += 2;
                }
            }
            // 4xkk - SNE Vx, byte: Skip next instruction if Vx != kk
            0x40 => {
                if self.vx[opcode.x()] != opcode.kk() {
                    next_pc += 2;
                }
            }
            // 5xy0 - SE Vx, Vy: Skip next instruction if Vx = Vy
            0x50 => {
                if self.vx[opcode.x()] == self.vx[opcode.y()] {
                    next_pc += 2;
                }
            }
            // 6xkk - LD Vx, byte: Set Vx = kk
            0x60 => {
                self.vx[opcode.x()] = opcode.kk();
            }
            // 7xkk - ADD Vx, byte: Set Vx = Vx + kk
            0x70 => {
                self.vx[opcode.x()] = self.vx[opcode.x()].wrapping_add(opcode.kk());
            }
            0x80 => match opcode.lower() & 0x0F {
                // 8xy0 - LD Vx, Vy: Set Vx = Vy
                0x00 => {
                    self.vx[opcode.x()] = self.vx[opcode.y()];
                }
                // 8xy1 - OR Vx, Vy: Set Vx = Vx OR Vy
                0x01 => {
                    self.vx[opcode.x()] |= self.vx[opcode.y()];
                }
                // 8xy2 - AND Vx, Vy: Set Vx = Vx AND Vy
                0x02 => {
                    self.vx[opcode.x()] &= self.vx[opcode.y()];
                }
                // 8xy3 - XOR Vx, Vy: Set Vx = Vx XOR Vy
                0x03 => {
                    self.vx[opcode.x()] ^= self.vx[opcode.y()];
                }
                // 8xy4 - ADD Vx, Vy: Set Vx = Vx + Vy, set VF = carry
                0x04 => {
                    let (vx, over) = self.vx[opcode.x()].overflowing_add(self.vx[opcode.y()]);
                    self.vx[opcode.x()] = vx;
                    self.vx[0xF] = over as u8;
                }
                // 8xy5 - SUB Vx, Vy: Set Vx = Vx - Vy, set VF = NOT borrow
                0x05 => {
                    let (vx, over) = self.vx[opcode.x()].overflowing_sub(self.vx[opcode.y()]);
                    self.vx[opcode.x()] = vx;
                    self.vx[0xF] = !over as u8;
                }
                // 8xy6 - SHR Vx {, Rhs}: Set Vx = Rhs SHR 1, set VF = LSB before shift
                // modified_shift = false: Rhs = Vy
                // modified_shift = true: Rhs = Vx
                0x06 => {
                    let rhs = self.vx[if self.config.modified_shift {
                        opcode.x()
                    } else {
                        opcode.y()
                    }];
                    self.vx[opcode.x()] = rhs >> 1;
                    self.vx[0xF] = rhs & 0x01;
                }
                // 8xy7 - SUBN Vx, Vy: Set Vx = Vy - Vx, set VF = NOT borrow
                0x07 => {
                    let (vx, over) = self.vx[opcode.y()].overflowing_sub(self.vx[opcode.x()]);
                    self.vx[opcode.x()] = vx;
                    self.vx[0xF] = !over as u8;
                }
                // 8xyE - SHL Vx {, Rhs}: Set Vx = Rhs SHL 1, set VF = MSB before shift
                // modified_shift = false: Rhs = Vy
                // modified_shift = true: Rhs = Vx
                0x0E => {
                    let rhs = self.vx[if self.config.modified_shift {
                        opcode.x()
                    } else {
                        opcode.y()
                    }];
                    self.vx[opcode.x()] = rhs << 1;
                    self.vx[0xF] = rhs >> 7;
                }
                _ => panic!("Invalid opcode 0x{:X}", *opcode),
            },
            // 9xy0 - SNE Vx, Vy: Skip next instruction if Vx != Vy
            0x90 => {
                if self.vx[opcode.x()] != self.vx[opcode.y()] {
                    next_pc += 2;
                }
            }

            // Annn - LD I, addr: Set I = nnn
            0xA0 => {
                self.i = opcode.nnn();
            }
            // Bnnn - JP V0, addr: Jump to location nnn + V0
            0xB0 => {
                next_pc = opcode.nnn() + self.vx[0] as u16;
            }
            // Cxkk - RND Vx, byte: Set Vx = random byte AND kk
            0xC0 => {
                self.vx[opcode.x()] = self.rng.u8(0..=255) & opcode.kk();
            }
            // Dxyn - DRW Vx, Vy, nibble: Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
            0xD0 => {
                let coords = (self.vx[opcode.x()], self.vx[opcode.y()]);
                let sprite = &self.memory[self.i as usize..self.i as usize + opcode.n() as usize];

                let collision = self.display.draw_sprite(coords, sprite);

                self.vx[0xF] = collision as u8;
            }
            0xE0 => match opcode.lower() {
                // Ex9E - SKP Vx: Skip next instruction if key with the value of Vx is pressed
                0x9E => match self.pressed.take() {
                    Some(key) if key == self.vx[opcode.x()] => next_pc += 2,
                    _ => (),
                },

                // ExA1 - SKNP Vx: Skip next instruction if key with the value of Vx is not pressed
                0xA1 => match self.pressed.take() {
                    Some(key) if key != self.vx[opcode.x()] => next_pc += 2,
                    _ => (),
                },

                _ => panic!("Invalid opcode 0x{:X}", *opcode),
            },
            0xF0 => match opcode.lower() {
                // Fx07 - LD Vx, DT: Set Vx = delay timer value
                0x07 => {
                    self.vx[opcode.x()] = self.timers.delay;
                }
                // Fx0A - LD Vx, K: Wait for a key press, store the value of the key in Vx
                0x0A => match self.pressed.take() {
                    Some(key) => self.vx[opcode.x()] = key as u8,
                    None => return,
                },
                // Fx15 - LD DT, Vx: Set delay timer = Vx
                0x15 => {
                    self.timers.delay = self.vx[opcode.x()];
                }
                // Fx18 - LD ST, Vx: Set sound timer = Vx
                0x18 => {
                    self.timers.sound = self.vx[opcode.x()];
                }
                // Fx1E - ADD I, Vx: Set I = I + Vx
                0x1E => {
                    self.i += self.vx[opcode.x()] as u16;
                }
                // Fx29 - LD F, Vx: Set I = location of sprite for digit Vx
                0x29 => {
                    self.i = self.vx[opcode.x()] as u16 * 5;
                }
                // Fx33 - LD B, Vx: Store BCD representation of Vx in memory locations I, I+1, and I+2
                0x33 => {
                    let mut vx = self.vx[opcode.x()];

                    self.memory[self.i as usize..=self.i as usize + 2]
                        .iter_mut()
                        .rev()
                        .for_each(|byte| {
                            *byte = vx % 10;
                            vx /= 10;
                        });
                }
                // Fx55 - LD [I], Vx: Store registers V0 through Vx in memory starting at location I
                // modified_load = false: Retain value of I
                // modified_load = true: Set I = I + x + 1
                0x55 => {
                    self.memory[self.i as usize..=self.i as usize + opcode.x() as usize]
                        .iter_mut()
                        .zip(self.vx.as_slice()[..=opcode.x() as usize].iter())
                        .for_each(|(byte, reg)| {
                            *byte = *reg;
                        });

                    if self.config.modified_load {
                        self.i += opcode.x() as u16 + 1;
                    }
                }
                // Fx65 - LD Vx, [I]: Read registers V0 through Vx from memory starting at location I
                // modified_load = false: Retain value of I
                // modified_load = true: Set I = I + x + 1
                0x65 => {
                    self.memory[self.i as usize..=self.i as usize + opcode.x() as usize]
                        .iter()
                        .zip(self.vx.as_mut_slice()[..=opcode.x() as usize].iter_mut())
                        .for_each(|(byte, reg)| {
                            *reg = *byte;
                        });
                    if self.config.modified_load {
                        self.i += opcode.x() as u16 + 1;
                    }
                }
                _ => panic!("Invalid opcode 0x{:X}", *opcode),
            },
            _ => panic!("Invalid opcode 0x{:X}", *opcode),
        }

        self.pc = next_pc;
    }
}

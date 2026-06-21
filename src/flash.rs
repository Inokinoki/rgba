//! GBA Flash Save Memory
//!
//! Supports Flash 64K and Flash 128K save types.
//! Flash uses a command sequence protocol accessed via memory-mapped I/O.

use std::collections::VecDeque;

/// Flash command state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum FlashState {
    Idle,
    CmdSequence1,
    CmdSequence2,
    CmdSelect,
    EraseSector,
    EraseSector2,
    Write,
    ReadId,
}

/// Flash memory chip (64KB or 128KB)
pub struct Flash {
    data: Vec<u8>,
    #[allow(dead_code)]
    size: usize,
    state: FlashState,
    cmd_buffer: VecDeque<u8>,
    #[allow(dead_code)]
    erase_sector_addr: u32,
    #[allow(dead_code)]
    write_addr: u32,
    id_mode: bool,
    bank: u8,
    pending_bank_select: bool,
}

impl Flash {
    pub fn new_64k() -> Self {
        Self {
            data: vec![0xFF; 0x10000], // 64KB
            size: 0x10000,
            state: FlashState::Idle,
            cmd_buffer: VecDeque::new(),
            erase_sector_addr: 0,
            write_addr: 0,
            id_mode: false,
            bank: 0,
            pending_bank_select: false,
        }
    }

    pub fn new_128k() -> Self {
        Self {
            data: vec![0xFF; 0x20000],
            size: 0x20000,
            state: FlashState::Idle,
            cmd_buffer: VecDeque::new(),
            erase_sector_addr: 0,
            write_addr: 0,
            id_mode: false,
            bank: 0,
            pending_bank_select: false,
        }
    }

    pub fn reset(&mut self) {
        self.state = FlashState::Idle;
        self.cmd_buffer.clear();
        self.id_mode = false;
        self.bank = 0;
    }

    /// Read a byte from flash
    pub fn read(&self, addr: u32) -> u8 {
        let addr = addr as usize & (self.size - 1);

        if self.id_mode {
            // Return chip ID bytes
            if self.size == 0x10000 {
                // 64K flash: manufacturer=0xC2, device=0x09 (SST39VF512)
                return match addr {
                    0 => 0xC2, // Manufacturer ID
                    1 => 0x09, // Device ID (64K)
                    _ => 0xFF,
                };
            } else {
                // 128K flash: manufacturer=0xC2, device=0x0D
                return match addr {
                    0 => 0xC2, // Manufacturer ID
                    1 => 0x0D, // Device ID (128K)
                    _ => 0xFF,
                };
            }
        }

        if self.size == 0x20000 && self.bank == 1 {
            // 128K flash bank 1
            self.data[0x10000 + (addr & 0xFFFF)]
        } else {
            self.data[addr & (self.size - 1)]
        }
    }

    /// Write a byte to flash (handles command sequences)
    pub fn write(&mut self, addr: u32, val: u8) {
        let raw_addr = addr as usize;

        match self.state {
            FlashState::Idle => {
                if val == 0xAA {
                    self.state = FlashState::CmdSequence1;
                }
            }
            FlashState::CmdSequence1 => {
                if val == 0x55 {
                    self.state = FlashState::CmdSequence2;
                } else {
                    self.state = FlashState::Idle;
                }
            }
            FlashState::CmdSequence2 => match val {
                0x90 => {
                    self.id_mode = true;
                    self.state = FlashState::Idle;
                }
                0xF0 => {
                    self.id_mode = false;
                    self.state = FlashState::Idle;
                }
                0x80 => {
                    self.state = FlashState::CmdSelect;
                }
                0xA0 => {
                    self.pending_bank_select = false;
                    self.state = FlashState::Write;
                }
                0xB0 => {
                    if self.size == 0x20000 {
                        self.pending_bank_select = true;
                        self.state = FlashState::Write;
                    } else {
                        self.state = FlashState::Idle;
                    }
                }
                _ => {
                    self.state = FlashState::Idle;
                }
            },
            FlashState::CmdSelect => match val {
                0xAA => {
                    self.state = FlashState::EraseSector;
                }
                0xA0 => {
                    self.pending_bank_select = false;
                    self.state = FlashState::Write;
                }
                _ => {
                    self.state = FlashState::Idle;
                }
            },
            FlashState::EraseSector => {
                if val == 0x55 {
                    self.state = FlashState::EraseSector2;
                } else {
                    self.state = FlashState::Idle;
                }
            }
            FlashState::EraseSector2 => {
                if val == 0x30 {
                    let sector_base = raw_addr & !0xFFF & (self.size - 1);
                    for b in &mut self.data[sector_base..sector_base + 0x1000] {
                        *b = 0xFF;
                    }
                } else if val == 0x10 {
                    self.data.fill(0xFF);
                }
                self.state = FlashState::Idle;
            }
            FlashState::Write => {
                if self.pending_bank_select {
                    self.bank = val & 1;
                    self.pending_bank_select = false;
                } else {
                    let target = if self.size == 0x20000 && self.bank == 1 {
                        0x10000 + (raw_addr & 0xFFFF)
                    } else {
                        raw_addr & (self.size - 1)
                    };
                    self.data[target] &= val;
                }
                self.state = FlashState::Idle;
            }
            FlashState::ReadId => {
                if val == 0xF0 {
                    self.id_mode = false;
                }
                self.state = FlashState::Idle;
            }
            _ => {
                self.state = FlashState::Idle;
            }
        }
    }

    /// Get a reference to the raw data (for saving)
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get a mutable reference to the raw data
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    /// Load data from a save file
    pub fn load(&mut self, data: &[u8]) {
        let len = data.len().min(self.data.len());
        self.data[..len].copy_from_slice(&data[..len]);
    }
}

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
    CmdSequence1, // After first unlock byte 0xAA
    CmdSequence2, // After second unlock byte 0x55
    CmdSelect,    // After command select byte
    EraseSector,  // Erasing sector
    Write,        // Writing bytes
    ReadId,       // Reading chip ID
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
    bank: u8, // For 128K flash, bank select
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
        }
    }

    pub fn new_128k() -> Self {
        Self {
            data: vec![0xFF; 0x20000], // 128KB
            size: 0x20000,
            state: FlashState::Idle,
            cmd_buffer: VecDeque::new(),
            erase_sector_addr: 0,
            write_addr: 0,
            id_mode: false,
            bank: 0,
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
        let addr = addr as usize & 0x0000_0001; // Flash commands use only A0

        match self.state {
            FlashState::Idle => {
                // First unlock byte: 0xAA to address 0x5555
                if val == 0xAA {
                    self.state = FlashState::CmdSequence1;
                }
            }
            FlashState::CmdSequence1 => {
                // Second unlock byte: 0x55 to address 0x2AAA
                if val == 0x55 {
                    self.state = FlashState::CmdSequence2;
                } else {
                    self.state = FlashState::Idle;
                }
            }
            FlashState::CmdSequence2 => {
                // Command select byte
                match val {
                    0x90 => {
                        // Enter ID mode
                        self.id_mode = true;
                        self.state = FlashState::Idle;
                    }
                    0xF0 => {
                        // Exit ID mode
                        self.id_mode = false;
                        self.state = FlashState::Idle;
                    }
                    0x80 => {
                        // Erase/Write enable
                        self.state = FlashState::CmdSelect;
                    }
                    0xB0 => {
                        // Bank select (128K only)
                        if self.size == 0x20000 {
                            self.state = FlashState::CmdSelect;
                        } else {
                            self.state = FlashState::Idle;
                        }
                    }
                    _ => {
                        self.state = FlashState::Idle;
                    }
                }
            }
            FlashState::CmdSelect => {
                // Sector erase or write
                match val {
                    0x30 => {
                        // Sector erase - erase 4KB sector
                        self.state = FlashState::Idle;
                    }
                    0x10 => {
                        // Chip erase
                        self.data.fill(0xFF);
                        self.state = FlashState::Idle;
                    }
                    0xA0 => {
                        // Write byte enable
                        self.state = FlashState::Write;
                    }
                    _ => {
                        self.state = FlashState::Idle;
                    }
                }
            }
            FlashState::Write => {
                // Write actual data byte
                let target = if self.size == 0x20000 && self.bank == 1 {
                    0x10000 + (addr & 0xFFFF)
                } else {
                    addr & (self.size - 1)
                };
                self.data[target] = val;
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

    /// Load data from a save file
    pub fn load(&mut self, data: &[u8]) {
        let len = data.len().min(self.data.len());
        self.data[..len].copy_from_slice(&data[..len]);
    }
}

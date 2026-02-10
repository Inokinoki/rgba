//! GBA Direct Memory Access Controller
//!
//! The GBA has 4 DMA channels that can transfer data between
//! memory regions without CPU intervention.

use crate::Memory;

/// DMA transfer mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaTransferMode {
    Immediate = 0,
    VBlank = 1,
    HBlank = 2,
    Special = 3,
}

/// DMA transfer type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaTransferType {
    HalfWord = 0,
    Word = 1,
}

/// GBA DMA Channel
pub struct Dma {
    num: u8,
    src_addr: u32,
    dst_addr: u32,
    count: u16,
    control: u16,
    enabled: bool,
    transfer_type: DmaTransferType,
    repeat: bool,
    src_increment: i8,  // -1, 0, or 1
    dst_increment: i8,  // -1, 0, or 1
    trigger: DmaTransferMode,
    irq: bool,
    active: bool,
    current_src: u32,
    current_dst: u32,
    current_count: u32, // Use u32 to handle 0x10000 for DMA3
}

impl Dma {
    pub fn new(num: u8) -> Self {
        Self {
            num,
            src_addr: 0,
            dst_addr: 0,
            count: 0,
            control: 0,
            enabled: false,
            transfer_type: DmaTransferType::HalfWord,
            repeat: false,
            src_increment: 0,
            dst_increment: 0,
            trigger: DmaTransferMode::Immediate,
            irq: false,
            active: false,
            current_src: 0,
            current_dst: 0,
            current_count: 0,
        }
    }

    pub fn reset(&mut self) {
        self.src_addr = 0;
        self.dst_addr = 0;
        self.count = 0;
        self.control = 0;
        self.enabled = false;
        self.active = false;
        self.transfer_type = DmaTransferType::HalfWord;
        self.repeat = false;
        self.src_increment = 0;
        self.dst_increment = 0;
        self.trigger = DmaTransferMode::Immediate;
        self.irq = false;
    }

    pub fn get_control(&self) -> u16 {
        self.control
    }

    pub fn set_control(&mut self, value: u16) {
        let was_enabled = self.enabled;
        self.control = value;

        // Decode control bits
        self.transfer_type = if (value & 0x0400) != 0 {
            DmaTransferType::Word
        } else {
            DmaTransferType::HalfWord
        };

        self.src_increment = match (value >> 7) & 0x3 {
            0 => 1,
            1 => -1,
            2 => 0,
            _ => 1, // Prohibited
        };

        self.dst_increment = match (value >> 5) & 0x3 {
            0 => 1,
            1 => -1,
            2 => 0,
            3 => 1, // Prohibited (except for DMA3 in special mode)
            _ => 1,
        };

        self.repeat = (value & 0x0200) != 0;
        self.trigger = match (value >> 12) & 0x3 {
            0 => DmaTransferMode::Immediate,
            1 => DmaTransferMode::VBlank,
            2 => DmaTransferMode::HBlank,
            _ => DmaTransferMode::Special,
        };

        self.irq = (value & 0x4000) != 0;
        self.enabled = (value & 0x8000) != 0;

        // If DMA is being enabled and wasn't before, initialize transfer
        if self.enabled && !was_enabled {
            self.start_transfer();
        }
    }

    fn start_transfer(&mut self) {
        self.current_src = self.src_addr;
        self.current_dst = self.dst_addr;
        self.current_count = if self.count == 0 {
            match self.num {
                3 => 0x10000,
                _ => 0x4000,
            }
        } else {
            self.count as u32
        };
        self.active = true;

        // For immediate transfer mode, execute immediately
        if self.trigger == DmaTransferMode::Immediate {
            // The transfer will be executed by the Gba::step function
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn get_trigger(&self) -> DmaTransferMode {
        self.trigger
    }

    pub fn get_src_addr(&self) -> u32 {
        self.src_addr
    }

    pub fn set_src_addr(&mut self, addr: u32) {
        self.src_addr = addr;
    }

    pub fn get_dst_addr(&self) -> u32 {
        self.dst_addr
    }

    pub fn set_dst_addr(&mut self, addr: u32) {
        self.dst_addr = addr;
    }

    pub fn get_count(&self) -> u16 {
        self.count
    }

    pub fn set_count(&mut self, count: u16) {
        self.count = count;
    }

    /// Execute the DMA transfer (called when trigger occurs)
    pub fn execute(&mut self, mem: &mut Memory) -> bool {
        if !self.active || !self.enabled {
            return false;
        }

        let transfer_size = match self.transfer_type {
            DmaTransferType::HalfWord => 2,
            DmaTransferType::Word => 4,
        };

        // Transfer data
        while self.current_count > 0 {
            match self.transfer_type {
                DmaTransferType::HalfWord => {
                    let value = mem.read_half(self.current_src);
                    mem.write_half(self.current_dst, value);
                }
                DmaTransferType::Word => {
                    let value = mem.read_word(self.current_src);
                    mem.write_word(self.current_dst, value);
                }
            }

            // Update addresses
            if self.src_increment > 0 {
                self.current_src = self.current_src.wrapping_add(transfer_size);
            } else if self.src_increment < 0 {
                self.current_src = self.current_src.wrapping_sub(transfer_size);
            }

            if self.dst_increment > 0 {
                self.current_dst = self.current_dst.wrapping_add(transfer_size);
            } else if self.dst_increment < 0 {
                self.current_dst = self.current_dst.wrapping_sub(transfer_size);
            }

            self.current_count -= 1;
        }

        // Check if DMA should repeat
        if self.repeat && self.trigger != DmaTransferMode::Immediate {
            self.current_src = self.src_addr;
            self.current_dst = self.dst_addr;
            self.current_count = if self.count == 0 {
                match self.num {
                    3 => 0x10000,
                    _ => 0x4000,
                }
            } else {
                self.count as u32
            };
            // Keep active for next trigger
        } else {
            // Transfer complete
            self.active = false;
            self.enabled = false;
            self.control &= !0x8000; // Clear enable bit

            // Return true if IRQ should be triggered
            return self.irq;
        }

        false
    }
}

impl Default for Dma {
    fn default() -> Self {
        Self::new(0)
    }
}

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
    src_increment: i8, // -1, 0, or 1
    dst_increment: i8, // -1, 0, or 1
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

        // DMAxCNT_H bit layout (GBATEK):
        // Bit 15:   DMA Enable (0=Off, 1=On)
        // Bit 14:   IRQ upon end of Word Count (0=Disable, 1=Enable)
        // Bit 13-12: DMA Start Timing (00=Immediately, 01=VBlank, 10=HBlank, 11=Special)
        // Bit 11:   Unused (should be 0)
        // Bit 10:   DMA Transfer Type (0=16bit, 1=32bit)
        // Bit 9:    DMA Repeat (0=Off, 1=On)
        // Bit 8-7:  Source Addr Control (00=Increment, 01=Decrement, 10=Fixed, 11=Prohibited)
        // Bit 6-5:  Dest Addr Control (00=Increment, 01=Decrement, 10=Fixed, 11=Increment/Reload)
        // Bit 4-0:  Unused (should be 0)

        self.enabled = (value & 0x8000) != 0;

        self.irq = (value & 0x4000) != 0;

        self.trigger = match (value >> 12) & 0x3 {
            0 => DmaTransferMode::Immediate,
            1 => DmaTransferMode::VBlank,
            2 => DmaTransferMode::HBlank,
            _ => DmaTransferMode::Special,
        };

        self.transfer_type = if (value & 0x0400) != 0 {
            DmaTransferType::Word
        } else {
            DmaTransferType::HalfWord
        };

        self.repeat = (value & 0x0200) != 0;

        self.src_increment = match (value >> 7) & 0x3 {
            0 => 1,
            1 => -1,
            2 => 0,
            _ => 1,
        };

        self.dst_increment = match (value >> 5) & 0x3 {
            0 => 1,
            1 => -1,
            2 => 0,
            3 => 1,
            _ => 1,
        };

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

    pub fn is_repeat(&self) -> bool {
        self.repeat
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn writeback_control(&self, io: &mut [u8]) {
        let base = 0xB0 + (self.num as usize * 12);
        io[base + 10] = self.control as u8;
        io[base + 11] = (self.control >> 8) as u8;
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

    pub fn execute(&mut self, mem: &mut Memory) -> bool {
        if !self.active || !self.enabled {
            return false;
        }

        mem.dma_active = true;

        let transfer_size = match self.transfer_type {
            DmaTransferType::HalfWord => 2,
            DmaTransferType::Word => 4,
        };

        if mem.dma_log_enabled {
            mem.dma_log.push((
                self.num,
                self.current_src,
                self.current_dst,
                self.current_count,
                transfer_size,
            ));
        }
        let dst_inc = self.dst_increment;
        let src_inc = self.src_increment;

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
            self.active = false;
            self.enabled = false;
            self.control &= !0x8000;
            mem.dma_active = false;
            return self.irq;
        }

        mem.dma_active = false;
        false
    }
}

impl Default for Dma {
    fn default() -> Self {
        Self::new(0)
    }
}

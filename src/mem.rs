//! GBA Memory System
//!
//! The GBA has a complex memory map with different regions having different
//! access timings and characteristics.
//!
//! Memory Map:
//! - 0x0000_0000 - 0x0000_3FFF: BIOS (16KB)
//! - 0x0200_0000 - 0x0203_FFFF: WRAM-B (256KB)
//! - 0x0300_0000 - 0x0300_7FFF: IWRAM (32KB)
//! - 0x0400_0000 - 0x0400_03FE: IO Registers
//! - 0x0500_0000 - 0x0500_03FF: Palette RAM (1KB)
//! - 0x0600_0000 - 0x0601_7FFF: VRAM (96KB)
//! - 0x0700_0000 - 0x0700_03FF: OAM (1KB)
//! - 0x0800_0000 - 0x0DFF_FFFF: ROM (max 32MB)

/// GBA Memory System
pub struct Memory {
    // BIOS ROM (16KB) - read-only after boot
    bios: Vec<u8>,

    // On-board Work RAM (256KB) - 3 cycles
    wram: Box<[u8; 0x40000]>,

    // On-chip Work RAM (32KB) - 1 cycle (fastest!)
    iwram: Box<[u8; 0x8000]>,

    // IO Registers (1KB)
    io: Box<[u8; 0x400]>,

    // Palette RAM (1KB) - 2KB actually, split BG/OBJ
    palette: Box<[u8; 0x400]>,

    // Video RAM (96KB) - can be accessed as 16-bit or 32-bit
    vram: Box<[u8; 0x18000]>,

    // Object Attribute Memory (1KB)
    oam: Box<[u8; 0x400]>,

    // ROM (max 32MB) - mirrored across different waitstate regions
    rom: Vec<u8>,

    // Waitstate configuration
    waitcnt: u16,
}

impl Memory {
    pub fn new() -> Self {
        // Initialize BIOS with default GBA BIOS
        // For now, just fill with zeros
        let bios = vec![0u8; 0x4000];

        Self {
            bios,
            wram: Box::new([0u8; 0x40000]),
            iwram: Box::new([0u8; 0x8000]),
            io: Box::new([0u8; 0x400]),
            palette: Box::new([0u8; 0x400]),
            vram: Box::new([0u8; 0x18000]),
            oam: Box::new([0u8; 0x400]),
            rom: Vec::new(),
            waitcnt: 0x0000,
        }
    }

    pub fn reset(&mut self) {
        self.wram.fill(0);
        self.iwram.fill(0);
        self.io.fill(0);
        self.palette.fill(0);
        self.vram.fill(0);
        self.oam.fill(0);
        self.waitcnt = 0x0000;
    }

    pub fn load_rom(&mut self, data: Vec<u8>) {
        self.rom = data;
    }

    /// Get access cycles for a memory region
    pub fn get_access_cycles(&self, addr: u32, _sequential: bool) -> u32 {
        match addr {
            0x0000_0000..=0x0000_3FFF => 2, // BIOS: always 2 cycles
            0x0200_0000..=0x0203_FFFF => 3, // WRAM: always 3 cycles
            0x0300_0000..=0x0300_7FFF => 1, // IWRAM: always 1 cycle
            0x0400_0000..=0x0400_03FE => 1, // IO: always 1 cycle
            0x0500_0000..=0x0500_03FF => 1, // Palette: always 1 cycle
            0x0600_0000..=0x0601_7FFF => 1, // VRAM: always 1 cycle
            0x0700_0000..=0x0700_03FF => 1, // OAM: always 1 cycle
            0x0800_0000..=0x09FF_FFFF => self.get_rom_waitstates(0, _sequential),
            0x0A00_0000..=0x0BFF_FFFF => self.get_rom_waitstates(1, _sequential),
            0x0C00_0000..=0x0DFF_FFFF => self.get_rom_waitstates(2, _sequential),
            _ => 1, // Unknown region
        }
    }

    fn get_rom_waitstates(&self, region: usize, sequential: bool) -> u32 {
        // Extract waitstate settings from WAITCNT register
        let ws = if sequential {
            match region {
                0 => (self.waitcnt >> 2) & 0x3,
                1 => (self.waitcnt >> 6) & 0x3,
                2 => (self.waitcnt >> 10) & 0x3,
                _ => 3,
            }
        } else {
            match region {
                0 => self.waitcnt & 0x3,
                1 => (self.waitcnt >> 4) & 0x3,
                2 => (self.waitcnt >> 8) & 0x3,
                _ => 3,
            }
        };

        // Convert waitstate to cycles (WS0=3, WS1=2, WS2=1, WS3=1? - varies)
        // Simplified: default to 3 cycles
        match ws {
            0 => 3,
            1 => 2,
            2 => 1,
            _ => 3,
        }
    }

    /// Map address to actual memory location
    fn map_address(&self, addr: u32) -> (MemoryRegion, usize) {
        match addr {
            0x0000_0000..=0x0000_3FFF => (MemoryRegion::Bios, (addr - 0x0000_0000) as usize),
            0x0200_0000..=0x0203_FFFF => (MemoryRegion::Wram, (addr - 0x0200_0000) as usize),
            0x0300_0000..=0x0300_7FFF => (MemoryRegion::Iwram, (addr - 0x0300_0000) as usize),
            0x0400_0000..=0x0400_03FE => (MemoryRegion::Io, (addr - 0x0400_0000) as usize),
            0x0500_0000..=0x0500_03FF => (MemoryRegion::Palette, (addr - 0x0500_0000) as usize),
            0x0600_0000..=0x0601_7FFF => (MemoryRegion::Vram, (addr - 0x0600_0000) as usize),
            0x0700_0000..=0x0700_03FF => (MemoryRegion::Oam, (addr - 0x0700_0000) as usize),
            0x0800_0000..=0x0DFF_FFFF => {
                let offset = (addr - 0x0800_0000) as usize;
                (MemoryRegion::Rom, offset % self.rom.len().max(1))
            }
            _ => (MemoryRegion::Unknown, 0),
        }
    }

    /// Read a byte from memory
    pub fn read_byte(&self, addr: u32) -> u8 {
        let (region, offset) = self.map_address(addr);

        match region {
            MemoryRegion::Bios => self.bios[offset],
            MemoryRegion::Wram => self.wram[offset],
            MemoryRegion::Iwram => self.iwram[offset],
            MemoryRegion::Io => self.read_io(addr),
            MemoryRegion::Palette => self.palette[offset],
            MemoryRegion::Vram => self.vram[offset],
            MemoryRegion::Oam => self.oam[offset],
            MemoryRegion::Rom => {
                if self.rom.is_empty() {
                    0
                } else {
                    self.rom[offset % self.rom.len()]
                }
            }
            MemoryRegion::Unknown => 0,
        }
    }

    /// Write a byte to memory
    pub fn write_byte(&mut self, addr: u32, val: u8) {
        let (region, offset) = self.map_address(addr);

        match region {
            MemoryRegion::Bios => {
                // BIOS is read-only
            }
            MemoryRegion::Wram => self.wram[offset] = val,
            MemoryRegion::Iwram => self.iwram[offset] = val,
            MemoryRegion::Io => self.write_io(addr, val),
            MemoryRegion::Palette => self.palette[offset] = val,
            MemoryRegion::Vram => self.vram[offset] = val,
            MemoryRegion::Oam => self.oam[offset] = val,
            MemoryRegion::Rom => {
                // ROM is read-only
            }
            MemoryRegion::Unknown => {}
        }
    }

    /// Read a halfword (16-bit) from memory
    pub fn read_half(&self, addr: u32) -> u16 {
        if addr & 1 != 0 {
            // Unaligned read - rotate
            let (region, offset) = self.map_address(addr & !1);
            let val = match region {
                MemoryRegion::Palette => u16::from_le_bytes([self.palette[offset], self.palette[offset + 1]]),
                MemoryRegion::Vram => u16::from_le_bytes([self.vram[offset], self.vram[offset + 1]]),
                MemoryRegion::Oam => u16::from_le_bytes([self.oam[offset], self.oam[offset + 1]]),
                _ => {
                    let low = self.read_byte(addr);
                    let high = self.read_byte(addr + 1);
                    u16::from_le_bytes([low, high])
                }
            };
            val.rotate_right(8 * (addr & 1) as u32)
        } else {
            let low = self.read_byte(addr);
            let high = self.read_byte(addr + 1);
            u16::from_le_bytes([low, high])
        }
    }

    /// Write a halfword (16-bit) to memory
    pub fn write_half(&mut self, addr: u32, val: u16) {
        let bytes = val.to_le_bytes();
        self.write_byte(addr, bytes[0]);
        self.write_byte(addr + 1, bytes[1]);
    }

    /// Read a word (32-bit) from memory
    pub fn read_word(&self, addr: u32) -> u32 {
        if addr & 3 != 0 {
            // Unaligned read - rotate
            let aligned = addr & !3;
            let low = self.read_half(aligned) as u32;
            let high = self.read_half(aligned + 2) as u32;
            let val = low | (high << 16);
            val.rotate_right(8 * (addr & 3) as u32)
        } else {
            let b0 = self.read_byte(addr) as u32;
            let b1 = self.read_byte(addr + 1) as u32;
            let b2 = self.read_byte(addr + 2) as u32;
            let b3 = self.read_byte(addr + 3) as u32;
            b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)
        }
    }

    /// Write a word (32-bit) to memory
    pub fn write_word(&mut self, addr: u32, val: u32) {
        let bytes = val.to_le_bytes();
        for i in 0..4usize {
            self.write_byte(addr + i as u32, bytes[i]);
        }
    }

    /// Read from IO register
    fn read_io(&self, addr: u32) -> u8 {
        let offset = (addr - 0x0400_0000) as usize;

        match offset {
            0x000 => self.io[offset] | 0x80, // DISPCNT - bit 7 is always set
            0x004 => self.io[offset], // DISPSTAT
            0x006 => self.io[offset], // VCOUNT (would be updated by PPU)
            0x130 => 0xFF, // Key input low byte - all keys released (active low, all 1s)
            0x131 => 0xFF, // Key input high byte - always 1
            _ => self.io[offset],
        }
    }

    /// Write to IO register
    fn write_io(&mut self, addr: u32, val: u8) {
        let offset = (addr - 0x0400_0000) as usize;

        match offset {
            0x204 => {
                // WAITCNT - only some bits are writable
                self.waitcnt = u16::from_le_bytes([val, self.io[offset + 1]]);
            }
            0x000..=0x003 => {
                // DISPCNT - display control
                self.io[offset] = val;
            }
            _ => {
                self.io[offset] = val;
            }
        }
    }

    /// Read a palette color entry (16-bit RGB555)
    /// pal_num: 0 for BG palette, 1 for OBJ palette
    /// index: color index (0-255)
    pub fn read_palette_color(&self, pal_num: usize, index: u16) -> u16 {
        // Palette RAM is at 0x0500_0000
        // BG palette: 0x0500_0000 - 0x0500_01FF (512 bytes, 256 colors)
        // OBJ palette: 0x0500_0200 - 0x0500_03FF (512 bytes, 256 colors)
        let offset = if pal_num == 0 {
            // BG palette
            (index as usize * 2) & 0x3FF
        } else {
            // OBJ palette (offset by 0x200)
            0x200 + ((index as usize * 2) & 0x1FF)
        };

        if offset + 1 < self.palette.len() {
            u16::from_le_bytes([self.palette[offset], self.palette[offset + 1]])
        } else {
            0
        }
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MemoryRegion {
    Bios,
    Wram,
    Iwram,
    Io,
    Palette,
    Vram,
    Oam,
    Rom,
    Unknown,
}

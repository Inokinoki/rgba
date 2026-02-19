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

use bitflags::bitflags;

bitflags! {
    /// Interrupt flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Interrupt: u16 {
        const VBLANK   = 0x0001;
        const HBLANK   = 0x0002;
        const VCOUNT   = 0x0004;
        const TIMER0   = 0x0008;
        const TIMER1   = 0x0010;
        const TIMER2   = 0x0020;
        const TIMER3   = 0x0040;
        const SERIAL   = 0x0080;
        const DMA0     = 0x0100;
        const DMA1     = 0x0200;
        const DMA2     = 0x0400;
        const DMA3     = 0x0800;
        const KEYPAD   = 0x1000;
        const GAMEPAK  = 0x2000;
    }
}

/// GBA Interrupt Controller (embedded in Memory for IO register handling)
pub struct InterruptController {
    /// Interrupt Enable register (0x0400_0200)
    pub ie: Interrupt,

    /// Interrupt Enable/FIRQ select (0x0400_0208)
    pub ie_fp: Interrupt,

    /// Interrupt Request flags (0x0400_0202)
    pub if_raw: Interrupt,

    /// Interrupt flags with masked disabled interrupts cleared
    if_processed: Interrupt,

    /// Interrupt Master Enable (0x0400_0208)
    pub ime: bool,

    /// Whether we're currently in an interrupt handler
    in_interrupt: bool,
}

impl InterruptController {
    pub fn new() -> Self {
        Self {
            ie: Interrupt::empty(),
            ie_fp: Interrupt::empty(),
            if_raw: Interrupt::empty(),
            if_processed: Interrupt::empty(),
            ime: false,
            in_interrupt: false,
        }
    }

    pub fn reset(&mut self) {
        self.ie = Interrupt::empty();
        self.ie_fp = Interrupt::empty();
        self.if_raw = Interrupt::empty();
        self.if_processed = Interrupt::empty();
        self.ime = false;
        self.in_interrupt = false;
    }

    /// Request an interrupt
    pub fn request(&mut self, interrupt: Interrupt) {
        self.if_raw |= interrupt;
    }

    /// Get pending interrupt (considering IE and IME)
    pub fn get_pending(&self) -> Option<Interrupt> {
        if !self.ime {
            return None;
        }

        // Get enabled interrupts that have requested
        let pending = self.ie & self.if_raw;

        if pending.is_empty() {
            None
        } else {
            // Return the highest priority interrupt (lowest bit number)
            let bit = pending.bits().trailing_zeros() as u16;
            Some(Interrupt::from_bits_truncate(1 << bit))
        }
    }

    /// Acknowledge an interrupt (clears IF bit)
    pub fn acknowledge(&mut self, interrupt: Interrupt) {
        self.if_raw &= !interrupt;
        self.if_processed &= !interrupt;
    }

    /// Check if we should take an interrupt
    pub fn should_take_interrupt(&self) -> bool {
        if !self.ime || self.in_interrupt {
            return false;
        }

        // Check if any enabled interrupt is pending
        !(self.ie & self.if_raw).is_empty()
    }

    /// Enter interrupt handler
    pub fn enter_interrupt(&mut self) {
        self.in_interrupt = true;
        self.ime = false; // IME is cleared on interrupt entry
    }

    /// Exit interrupt handler
    pub fn exit_interrupt(&mut self) {
        self.in_interrupt = false;
        self.ime = true; // IME is restored on interrupt exit
    }

    /// Read IO register
    pub fn read_register(&self, offset: usize) -> u16 {
        match offset {
            0x000 => self.ie.bits(),
            0x002 => self.if_raw.bits(),
            0x200 => self.ie_fp.bits(),
            0x208 => self.ime as u16,
            _ => 0,
        }
    }

    /// Write IO register
    pub fn write_register(&mut self, offset: usize, val: u16) {
        match offset {
            0x000 => self.ie = Interrupt::from_bits_truncate(val),
            0x002 => {
                // IF - writing 1 clears the bit, writing 0 has no effect
                self.if_raw &= !(Interrupt::from_bits_truncate(val));
                self.if_processed &= !(Interrupt::from_bits_truncate(val));
            }
            0x200 => self.ie_fp = Interrupt::from_bits_truncate(val),
            0x208 => self.ime = val != 0,
            _ => {}
        }
    }
}

impl Default for InterruptController {
    fn default() -> Self {
        Self::new()
    }
}

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

    // Interrupt controller
    pub interrupt: InterruptController,
}

impl Memory {
    pub fn new() -> Self {
        // Initialize BIOS with stub implementation
        // Fill with ARM NOP (0xE1A00000) and function returns at key locations
        let mut bios = vec![0u8; 0x4000];

        // Fill most of BIOS with ARM NOP (0xE1A00000 in little endian)
        for i in (0..0x4000).step_by(4) {
            bios[i] = 0x00;
            bios[i + 1] = 0x00;
            bios[i + 2] = 0xA0;
            bios[i + 3] = 0xE1;
        }

        // At BIOS entry point (0x00000000), we need to:
        // 1. Set up SP properly (0x03007F00 for IRQ mode, 0x03007FE0 for SVC mode)
        // 2. Do other initialization
        // 3. Jump to ROM at 0x08000000
        //
        // For now, just jump to ROM directly
        // B 0x08000004 (jump to ROM entry + skip header)
        // 0xEA00003E in little endian: 3E 00 00 EA
        bios[0] = 0x3E;
        bios[1] = 0x00;
        bios[2] = 0x00;
        bios[3] = 0xEA;

        // At key BIOS entry points used by tests, put "BX LR" or "MOV PC, LR" to return
        // BX LR in ARM: 0xE12FFF1E
        let bios_return: [u8; 4] = [0x1E, 0xFF, 0x2F, 0xE1];

        // Set returns at common BIOS call points
        for offset in [0x164, 0x17C, 0x200, 0x208].iter() {
            if *offset + 4 <= 0x4000 {
                bios[*offset..(*offset + 4)].copy_from_slice(&bios_return);
            }
        }

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
            interrupt: InterruptController::new(),
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
        self.interrupt.reset();
    }

    pub fn load_rom(&mut self, data: Vec<u8>) {
        self.rom = data;
    }

    /// Load BIOS from a file
    pub fn load_bios(&mut self, data: Vec<u8>) {
        // BIOS is 16KB, truncate or pad as needed
        let mut bios_data = vec![0u8; 0x4000];
        let len = data.len().min(0x4000);
        bios_data[..len].copy_from_slice(&data[..len]);
        self.bios = bios_data;
    }

    /// Check if BIOS is loaded (not all zeros)
    pub fn has_bios(&self) -> bool {
        self.bios.iter().any(|&b| b != 0)
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
            // EWRAM (256KB) and its mirrors
            0x0200_0000..=0x020F_FFFF => {
                let offset = ((addr - 0x0200_0000) & 0x3_FFFF) as usize; // Mask to 256KB
                (MemoryRegion::Wram, offset)
            }
            // IWRAM (32KB) and its mirrors
            0x0300_0000..=0x030F_FFFF => {
                let offset = ((addr - 0x0300_0000) & 0x7FFF) as usize; // Mask to 32KB
                (MemoryRegion::Iwram, offset)
            }
            0x0400_0000..=0x0400_03FE => (MemoryRegion::Io, (addr - 0x0400_0000) as usize),
            // Palette (1KB) and its mirrors
            0x0500_0000..=0x0500_03FF => (MemoryRegion::Palette, (addr - 0x0500_0000) as usize),
            0x0500_0400..=0x050F_FFFF => {
                let offset = ((addr - 0x0500_0000) & 0x3FF) as usize; // Mask to 1KB
                (MemoryRegion::Palette, offset)
            }
            // VRAM (96KB) and its mirrors
            0x0600_0000..=0x060F_FFFF => {
                let offset = ((addr - 0x0600_0000) & 0x1_7FFF) as usize; // Mask to 96KB
                (MemoryRegion::Vram, offset)
            }
            // OAM (1KB) and its mirrors
            0x0700_0000..=0x0700_03FF => (MemoryRegion::Oam, (addr - 0x0700_0000) as usize),
            0x0700_0400..=0x070F_FFFF => {
                let offset = ((addr - 0x0700_0000) & 0x3FF) as usize; // Mask to 1KB
                (MemoryRegion::Oam, offset)
            }
            // ROM (with mirrors)
            0x0800_0000..=0x09FF_FFFF => {
                let offset = (addr - 0x0800_0000) as usize;
                (MemoryRegion::Rom, offset % self.rom.len().max(1))
            }
            // ROM mirror at 0x0A000000-0x0BFFFFFF (GamePak mirror 1)
            0x0A00_0000..=0x0BFF_FFFF => {
                let offset = (addr - 0x0A00_0000) as usize;
                (MemoryRegion::Rom, offset % self.rom.len().max(1))
            }
            // ROM mirror at 0x0C000000-0x0DFFFFFF (GamePak mirror 2)
            0x0C00_0000..=0x0DFF_FFFF => {
                let offset = (addr - 0x0C00_0000) as usize;
                (MemoryRegion::Rom, offset % self.rom.len().max(1))
            }
            _ => (MemoryRegion::Unknown, 0),
        }
    }

    /// Read a byte from memory
    pub fn read_byte(&mut self, addr: u32) -> u8 {
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

    /// Write a byte to memory (internal, used by write_word)
    fn write_byte_internal(&mut self, addr: u32, val: u8) {
        let (region, offset) = self.map_address(addr);

        match region {
            MemoryRegion::Bios => {
                // BIOS is read-only
            }
            MemoryRegion::Wram => self.wram[offset] = val,
            MemoryRegion::Iwram => {
                self.iwram[offset] = val;
            }
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

    /// Write a byte to memory (public, handles OAM and VRAM byte-write restrictions)
    pub fn write_byte(&mut self, addr: u32, val: u8) {
        let (region, offset) = self.map_address(addr);

        // OAM ignores byte writes (only accepts 16-bit or 32-bit aligned writes)
        if region == MemoryRegion::Oam {
            return;
        }

        // VRAM: byte writes are expanded to halfwords (duplicated in both bytes)
        // This happens in ALL modes according to GBA behavior
        if region == MemoryRegion::Vram {
            let half_offset = offset & !1; // Align to halfword boundary
            let half_val = ((val as u16) << 8) | (val as u16); // Duplicate byte
            self.vram[half_offset] = (half_val & 0xFF) as u8;
            self.vram[half_offset + 1] = ((half_val >> 8) & 0xFF) as u8;
            return;
        }

        // Palette RAM: byte writes are expanded to halfwords (duplicated in both bytes)
        if region == MemoryRegion::Palette {
            let half_offset = offset & !1; // Align to halfword boundary
            let half_val = ((val as u16) << 8) | (val as u16); // Duplicate byte
            self.palette[half_offset] = (half_val & 0xFF) as u8;
            self.palette[half_offset + 1] = ((half_val >> 8) & 0xFF) as u8;
            return;
        }

        self.write_byte_internal(addr, val);
    }

    /// Read a halfword (16-bit) from memory
    pub fn read_half(&mut self, addr: u32) -> u16 {
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
        self.write_byte_internal(addr, bytes[0]);
        self.write_byte_internal(addr + 1, bytes[1]);
    }

    /// Read a word (32-bit) from memory
    pub fn read_word(&mut self, addr: u32) -> u32 {
        if addr & 3 != 0 {
            // Unaligned read - rotate
            let aligned = addr & !3;
            let low = self.read_half(aligned) as u32;
            let high = self.read_half(aligned.wrapping_add(2)) as u32;
            let val = low | (high << 16);
            val.rotate_right(8 * (addr & 3) as u32)
        } else {
            let b0 = self.read_byte(addr) as u32;
            let b1 = self.read_byte(addr.wrapping_add(1)) as u32;
            let b2 = self.read_byte(addr.wrapping_add(2)) as u32;
            let b3 = self.read_byte(addr.wrapping_add(3)) as u32;
            b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)
        }
    }

    /// Write a word (32-bit) to memory
    pub fn write_word(&mut self, addr: u32, val: u32) {
        let bytes = val.to_le_bytes();
        for i in 0..4usize {
            self.write_byte_internal(addr.wrapping_add(i as u32), bytes[i]);
        }
    }

    /// Read from IO register
    fn read_io(&mut self, addr: u32) -> u8 {
        let offset = (addr - 0x0400_0000) as usize;

        // Handle interrupt registers (IE is at 0x0400_0200, not 0x0400_0000!)
        let (int_offset, byte_index) = match addr {
            0x0400_0200 | 0x0400_0201 => (Some(0x200), (addr & 1) as usize), // IE
            0x0400_0202 | 0x0400_0203 => (Some(0x202), (addr & 1) as usize), // IF
            0x0400_0208 => (Some(0x208), 0), // IME
            _ => (None, 0),
        };

        if let Some(ioff) = int_offset {
            let val = self.interrupt.read_register(ioff);
            return if ioff == 0x208 {
                val as u8
            } else {
                (val >> (8 * byte_index as u32)) as u8
            };
        }

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

        // Handle interrupt registers (IE is at 0x0400_0200, not 0x0400_0000!)
        let (int_offset, byte_index) = match addr {
            0x0400_0200 | 0x0400_0201 => (Some(0x200), (addr & 1) as usize), // IE
            0x0400_0202 | 0x0400_0203 => (Some(0x202), (addr & 1) as usize), // IF
            0x0400_0208 => (Some(0x208), 0), // IME
            _ => (None, 0),
        };

        if let Some(ioff) = int_offset {
            // Read current value, modify the byte, write back
            let current = self.interrupt.read_register(ioff);
            let new_val = if ioff == 0x208 {
                val as u16
            } else {
                let shift = 8 * byte_index as u32;
                let mask = 0xFF << shift;
                let current_cleared = current & !mask;
                current_cleared | ((val as u16) << shift)
            };
            self.interrupt.write_register(ioff, new_val);
            return;
        }

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

    /// Get a reference to VRAM data
    pub fn vram(&self) -> &[u8] {
        &self.vram[..]
    }

    /// Get a reference to IO register data
    pub fn io(&self) -> &[u8] {
        &self.io[..]
    }

    /// Get a mutable reference to IO register data
    pub fn io_mut(&mut self) -> &mut [u8] {
        &mut self.io[..]
    }

    /// Check if address is in interrupt register range (0x0400_0000 - 0x0400_0208)
    pub fn is_interrupt_register(addr: u32) -> bool {
        matches!(addr, 0x0400_0000..=0x0400_0208)
    }

    /// Get interrupt register offset (returns 0xFFFF if not in range)
    pub fn get_interrupt_register_offset(addr: u32) -> Option<usize> {
        match addr {
            0x0400_0000 | 0x0400_0001 => Some(0x000), // IE
            0x0400_0002 | 0x0400_0003 => Some(0x002), // IF
            0x0400_0208 => Some(0x208), // IME
            _ => None,
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

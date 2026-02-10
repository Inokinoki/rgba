mod cpu;
mod mem;
mod ppu;
mod apu;
mod timer;
mod dma;
mod input;

pub use cpu::Cpu;
pub use mem::Memory;
pub use ppu::Ppu;
pub use apu::Apu;
pub use timer::Timer;
pub use dma::Dma;
pub use input::{Input, KeyState};

use std::fmt;

/// Represents the GBA console
pub struct Gba {
    pub cpu: Cpu,
    pub mem: Memory,
    pub ppu: Ppu,
    pub apu: Apu,
    pub timers: [Timer; 4],
    pub dma: [Dma; 4],
    pub input: Input,
}

impl Gba {
    /// Creates a new GBA instance
    pub fn new() -> Self {
        let mut gba = Self {
            cpu: Cpu::new(),
            mem: Memory::new(),
            ppu: Ppu::new(),
            apu: Apu::new(),
            timers: [
                Timer::new(0),
                Timer::new(1),
                Timer::new(2),
                Timer::new(3),
            ],
            dma: [
                Dma::new(0),
                Dma::new(1),
                Dma::new(2),
                Dma::new(3),
            ],
            input: Input::new(),
        };
        gba.cpu.reset(); // Initialize CPU to proper GBA state
        gba
    }

    /// Resets the GBA to its initial state
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.mem.reset();
        self.ppu.reset();
        self.apu.reset();
        for timer in &mut self.timers {
            timer.reset();
        }
        for dma in &mut self.dma {
            dma.reset();
        }
        self.input.reset();
    }

    /// Runs the emulator for one frame
    pub fn run_frame(&mut self) {
        // GBA runs at ~16.78 MHz
        // Each frame is 280896 cycles (59.57 Hz)
        for _ in 0..280896 {
            self.step();
        }
    }

    /// Executes a single step
    pub fn step(&mut self) {
        let cycles = self.cpu.step(&mut self.mem);
        self.ppu.step(cycles);
        self.apu.step(cycles);
        for timer in &mut self.timers {
            timer.step(cycles);
        }
    }

    /// Loads a ROM into memory
    pub fn load_rom(&mut self, data: Vec<u8>) {
        self.mem.load_rom(data);
    }

    /// Loads a ROM from a file path
    pub fn load_rom_path(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;
        use std::io::Read;

        let mut file = fs::File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        self.load_rom(data);
        Ok(())
    }

    /// Load BIOS from a file path
    pub fn load_bios_path(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;
        use std::io::Read;

        let mut file = fs::File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        self.mem.load_bios(data);
        Ok(())
    }

    /// Get a reference to the PPU
    pub fn ppu(&self) -> &Ppu {
        &self.ppu
    }

    /// Get a mutable reference to the PPU
    pub fn ppu_mut(&mut self) -> &mut Ppu {
        &mut self.ppu
    }

    /// Sync PPU state from Memory (IO registers and VRAM)
    /// This must be called before rendering to get the latest state
    pub fn sync_ppu(&mut self) {
        // Sync VRAM
        self.ppu.sync_vram(self.mem.vram());

        // Sync IO registers
        let io = self.mem.io();

        // DISPCNT (0x0400_0000)
        let dispcnt = u16::from_le_bytes([io[0], io[1]]);
        self.ppu.set_display_enabled((dispcnt & 0x80) != 0);
        self.ppu.set_display_mode((dispcnt & 0x7) as u8);

        // BG0CNT - BG3CNT (0x0400_0008 - 0x0400_000E)
        for bg in 0..4 {
            let offset = 8 + (bg * 2);
            let bgcnt = u16::from_le_bytes([io[offset], io[offset + 1]]);
            self.ppu.set_bgcnt(bg, bgcnt);

            // Enable/disable BG based on DISPCNT bits 8-11
            let bg_enabled = (dispcnt & (0x100 << bg)) != 0;
            self.ppu.set_bg_enabled(bg, bg_enabled);
        }

        // BG0HOFS - BG3VOFS (0x0400_0010 - 0x0400_002D)
        for bg in 0..4 {
            let hofs_offset = 16 + (bg * 4);
            let vofs_offset = hofs_offset + 2;
            let hofs = u16::from_le_bytes([io[hofs_offset], io[hofs_offset + 1]]) & 0x1FF;
            let vofs = u16::from_le_bytes([io[vofs_offset], io[vofs_offset + 1]]) & 0x1FF;
            self.ppu.set_bg_hofs(bg, hofs);
            self.ppu.set_bg_vofs(bg, vofs);
        }

        // BLDCNT (0x0400_0050)
        let bldcnt = u16::from_le_bytes([io[0x50], io[0x51]]);
        self.ppu.set_blend_control(bldcnt);

        // BLDALPHA (0x0400_0052)
        let bldalpha = u16::from_le_bytes([io[0x52], io[0x53]]);
        self.ppu.set_blend_alpha(bldalpha);
    }

    /// Get a mutable reference to the input system
    pub fn input_mut(&mut self) -> &mut Input {
        &mut self.input
    }

    /// Get a reference to the CPU
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    /// Get a reference to the memory system (for palette access)
    pub fn mem(&self) -> &Memory {
        &self.mem
    }

    /// Get a mutable reference to the memory system
    pub fn mem_mut(&mut self) -> &mut Memory {
        &mut self.mem
    }

    /// Write a byte to memory (for testing/debugging)
    pub fn write_byte(&mut self, addr: u32, val: u8) {
        self.mem.write_byte(addr, val);
    }

    /// Write a halfword (16-bit) to memory
    pub fn write_half(&mut self, addr: u32, val: u16) {
        self.mem.write_half(addr, val);
    }

    /// Write a word (32-bit) to memory
    pub fn write_word(&mut self, addr: u32, val: u32) {
        self.mem.write_word(addr, val);
    }

    /// Read a byte from memory
    pub fn read_byte(&self, addr: u32) -> u8 {
        self.mem.read_byte(addr)
    }

    /// Read palette color (RGB555) from palette RAM
    /// pal_num: 0 for BG palette, 1 for OBJ palette
    /// index: color index (0-255)
    pub fn get_palette_color(&self, pal_num: usize, index: u16) -> u16 {
        self.mem.read_palette_color(pal_num, index)
    }

    /// Get pixel color for tile/text modes (0, 1, 2)
    /// Returns RGB555 color value
    pub fn get_pixel_tile_mode(&self, x: u16, y: u16) -> u16 {
        let ppu = &self.ppu;
        let mode = ppu.get_display_mode();

        match mode {
            0 | 1 | 2 => {
                // Tile/text modes - render backgrounds from lowest to highest priority
                let mut color = 0; // Default: transparent (black)

                // Check each background layer (BG0-BG3)
                for bg in 0..4 {
                    if ppu.is_bg_enabled(bg) {
                        // Get BG control register
                        let bgcnt = ppu.get_bgcnt(bg);
                        let priority = ppu.get_bg_priority(bg);

                        // Background size encoding
                        let bg_size = (bgcnt >> 14) & 0x3;

                        // Get dimensions based on size and mode
                        let (width, height) = match (mode, bg_size) {
                            // Regular BG (BG0, BG1 in modes 0, 1)
                            (_, 0) => (256u16, 256u16),
                            (_, 1) => (512u16, 256u16),
                            (_, 2) => (256u16, 512u16),
                            (_, 3) => (512u16, 512u16),
                            _ => (256u16, 256u16),
                        };

                        // Affine BG (BG2, BG3 in mode 2) use different dimensions
                        let (width, height) = if mode == 2 && (bg == 2 || bg == 3) {
                            match bg_size {
                                0 => (128u16, 128u16),
                                1 => (256u16, 256u16),
                                2 => (512u16, 512u16),
                                3 => (1024u16, 1024u16),
                                _ => (128u16, 128u16),
                            }
                        } else {
                            (width, height)
                        };

                        // Apply scroll offset
                        let hofs = ppu.get_bg_hofs(bg);
                        let vofs = ppu.get_bg_vofs(bg);
                        let bg_x = ((x as u32 + hofs as u32) % width as u32) as u16;
                        let bg_y = ((y as u32 + vofs as u32) % height as u32) as u16;

                        // Get tile coordinates
                        let tile_x = bg_x / 8;
                        let tile_y = bg_y / 8;
                        let pixel_x = bg_x % 8;
                        let pixel_y = bg_y % 8;

                        // Get screen entry (tile map entry)
                        let screen_base = ppu.get_bg_map_base(bg) as usize;
                        let entry = ppu.get_screen_entry(
                            screen_base,
                            tile_x,
                            tile_y,
                            bg_size,
                            width / 8,
                            height / 8,
                        );

                        // Parse screen entry
                        let (tile_num, flip_h, flip_v, palette_num, _priority) =
                            Ppu::parse_screen_entry(entry);

                        // Check if 8bpp or 4bpp
                        let is_8bpp = (bgcnt & 0x80) != 0;

                        // Get tile data
                        let tile_base = ppu.get_bg_tile_base(bg) as usize;

                        let color_index = if is_8bpp {
                            ppu.get_tile_pixel_8bpp(
                                tile_base,
                                tile_num,
                                pixel_x as u8,
                                pixel_y as u8,
                                flip_h,
                                flip_v,
                            )
                        } else {
                            ppu.get_tile_pixel_4bpp(
                                tile_base,
                                tile_num,
                                pixel_x as u8,
                                pixel_y as u8,
                                palette_num,
                                flip_h,
                                flip_v,
                            )
                        };

                        // If not transparent (0), use this color
                        if color_index != 0 {
                            // Get actual palette color
                            let pal_index = if is_8bpp {
                                color_index as u16
                            } else {
                                (palette_num * 16) + color_index as u16
                            };
                            color = self.get_palette_color(0, pal_index);

                            // For now, take the first non-transparent pixel
                            // In a full implementation, we'd layer by priority
                            return color;
                        }
                    }
                }

                color
            }
            _ => 0, // Other modes handled elsewhere
        }
    }
}

impl Default for Gba {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Gba {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Gba")
            .field("cpu", &self.cpu)
            .finish()
    }
}

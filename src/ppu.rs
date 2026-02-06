//! GBA Picture Processing Unit (PPU)
//!
//! Handles all graphics rendering including:
//! - Display modes 0-5 (tile modes and bitmap modes)
//! - Background layers (up to 4)
//! - Sprite (OBJ) rendering
//! - Special effects (mosaic, alpha blending, windowing)

use bitflags::bitflags;

bitflags! {
    /// Display control flags (DISPCNT)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DisplayControl: u16 {
        const MODE0 = 0 << 0;
        const MODE1 = 1 << 0;
        const MODE2 = 2 << 0;
        const MODE3 = 3 << 0;
        const MODE4 = 4 << 0;
        const MODE5 = 5 << 0;
        const MODE_MASK = 0b111;

        const BG0 = 1 << 8;
        const BG1 = 1 << 9;
        const BG2 = 1 << 10;
        const BG3 = 1 << 11;
        const OBJ = 1 << 12;
        const WIN0 = 1 << 13;
        const WIN1 = 1 << 14;
        const OBJ_WIN = 1 << 15;

        const FRAME_0 = 0 << 4;
        const FRAME_1 = 1 << 4;
    }
}

/// GBA Picture Processing Unit
pub struct Ppu {
    // Display control
    dispcnt: DisplayControl,
    display_enabled: bool,

    // Display status
    dispstat: u16,

    // Current scanline
    vcount: u16,

    // HBlank counter (0-1232)
    hcounter: u32,

    // Background control
    bgcnt: [u16; 4],

    // Background offsets and transformations
    bg_hofs: [u16; 4],
    bg_vofs: [u16; 4],
    bg_affine: [[u32; 4]; 2], // For BG2 and BG3

    // Mosaic settings
    bg_mosaic: u16,
    obj_mosaic: u16,

    // Window settings
    win0_h: u16,
    win0_v: u16,
    win1_h: u16,
    win1_v: u16,
    winin: u16,
    winout: u16,

    // Blending settings
    bldcnt: u16,
    bldalpha: u16,
    bldy: u16,

    // Reference to memory for VRAM, palette, OAM access
    // (In real implementation, this would be shared)
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            dispcnt: DisplayControl::empty(),
            display_enabled: false,
            dispstat: 0,
            vcount: 0,
            hcounter: 0,
            bgcnt: [0; 4],
            bg_hofs: [0; 4],
            bg_vofs: [0; 4],
            bg_affine: [[0x100, 0, 0, 0x100], [0x100, 0, 0, 0x100]], // Identity matrices
            bg_mosaic: 0,
            obj_mosaic: 0,
            win0_h: 0,
            win0_v: 0,
            win1_h: 0,
            win1_v: 0,
            winin: 0,
            winout: 0,
            bldcnt: 0,
            bldalpha: 0,
            bldy: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    // Display control
    pub fn is_display_enabled(&self) -> bool {
        self.display_enabled
    }

    pub fn set_display_enabled(&mut self, enabled: bool) {
        self.display_enabled = enabled;
    }

    pub fn get_display_mode(&self) -> u8 {
        (self.dispcnt.bits() & 0x7) as u8
    }

    pub fn set_display_mode(&mut self, mode: u8) {
        let bits = self.dispcnt.bits() & !0x7;
        self.dispcnt = DisplayControl::from_bits_truncate(bits | (mode as u16 & 0x7));
    }

    pub fn get_width(&self) -> u16 {
        match self.get_display_mode() {
            3 | 4 => 240,
            5 => 160,
            _ => 240,
        }
    }

    pub fn get_height(&self) -> u16 {
        match self.get_display_mode() {
            5 => 128,
            _ => 160,
        }
    }

    // Scanline state
    pub fn get_vcount(&self) -> u16 {
        self.vcount
    }

    pub fn set_vcount(&mut self, count: u16) {
        self.vcount = count;
    }

    pub fn set_hcounter(&mut self, count: u32) {
        self.hcounter = count;
    }

    pub fn is_in_vblank(&self) -> bool {
        self.vcount >= 160
    }

    pub fn is_in_hblank(&self) -> bool {
        self.hcounter >= 960 // Visible pixels: 240 * 4 = 960 cycles
    }

    // Background control
    pub fn is_bg_enabled(&self, bg: usize) -> bool {
        if bg > 3 {
            return false;
        }
        self.dispcnt.contains(DisplayControl::from_bits_truncate(1 << (8 + bg)))
    }

    pub fn set_bg_enabled(&mut self, bg: usize, enabled: bool) {
        if bg > 3 {
            return;
        }
        let flag = DisplayControl::from_bits_truncate(1 << (8 + bg));
        if enabled {
            self.dispcnt |= flag;
        } else {
            self.dispcnt &= !flag;
        }
    }

    pub fn get_bg_priority(&self, bg: usize) -> u16 {
        if bg > 3 {
            return 0;
        }
        self.bgcnt[bg] & 0x3
    }

    pub fn set_bg_priority(&mut self, bg: usize, priority: u16) {
        if bg > 3 {
            return;
        }
        self.bgcnt[bg] = (self.bgcnt[bg] & !0x3) | (priority & 0x3);
    }

    pub fn get_bg_tile_base(&self, bg: usize) -> u16 {
        if bg > 3 {
            return 0;
        }
        ((self.bgcnt[bg] >> 2) & 0x3) * 16 // In 16KB units
    }

    pub fn set_bg_tile_base(&mut self, bg: usize, base: u16) {
        if bg > 3 {
            return;
        }
        self.bgcnt[bg] = (self.bgcnt[bg] & !0xC) | ((base & 0x60) << 2);
    }

    pub fn get_bg_map_base(&self, bg: usize) -> u16 {
        if bg > 3 {
            return 0;
        }
        ((self.bgcnt[bg] >> 8) & 0x1F) * 2 // In 2KB units
    }

    pub fn set_bg_map_base(&mut self, bg: usize, base: u16) {
        if bg > 3 {
            return;
        }
        self.bgcnt[bg] = (self.bgcnt[bg] & !0x1F00) | ((base & 0x1F) << 8);
    }

    // Affine background
    pub fn set_bg_affine_a(&mut self, bg: usize, val: u32) {
        if bg == 2 || bg == 3 {
            self.bg_affine[bg - 2][0] = val;
        }
    }

    pub fn get_bg_affine_a(&self, bg: usize) -> u32 {
        if bg == 2 || bg == 3 {
            self.bg_affine[bg - 2][0]
        } else {
            0x100
        }
    }

    pub fn set_bg_affine_b(&mut self, bg: usize, val: u32) {
        if bg == 2 || bg == 3 {
            self.bg_affine[bg - 2][1] = val;
        }
    }

    pub fn set_bg_affine_c(&mut self, bg: usize, val: u32) {
        if bg == 2 || bg == 3 {
            self.bg_affine[bg - 2][2] = val;
        }
    }

    pub fn set_bg_affine_d(&mut self, bg: usize, val: u32) {
        if bg == 2 || bg == 3 {
            self.bg_affine[bg - 2][3] = val;
        }
    }

    // Mosaic
    pub fn get_bg_mosaic_h(&self) -> u16 {
        (self.bg_mosaic & 0xF) + 1
    }

    pub fn set_bg_mosaic_h(&mut self, val: u16) {
        self.bg_mosaic = (self.bg_mosaic & !0xF) | (val - 1).min(0xF);
    }

    pub fn get_bg_mosaic_v(&self) -> u16 {
        ((self.bg_mosaic >> 4) & 0xF) + 1
    }

    pub fn set_bg_mosaic_v(&mut self, val: u16) {
        self.bg_mosaic = (self.bg_mosaic & !0xF0) | ((val - 1).min(0xF) << 4);
    }

    pub fn get_obj_mosaic_h(&self) -> u16 {
        (self.obj_mosaic & 0xF) + 1
    }

    pub fn set_obj_mosaic_h(&mut self, val: u16) {
        self.obj_mosaic = (self.obj_mosaic & !0xF) | (val - 1).min(0xF);
    }

    pub fn get_obj_mosaic_v(&self) -> u16 {
        ((self.obj_mosaic >> 4) & 0xF) + 1
    }

    pub fn set_obj_mosaic_v(&mut self, val: u16) {
        self.obj_mosaic = (self.obj_mosaic & !0xF0) | ((val - 1).min(0xF) << 4);
    }

    // Window
    pub fn is_window1_enabled(&self) -> bool {
        self.dispcnt.contains(DisplayControl::WIN1)
    }

    pub fn set_window1_enabled(&mut self, enabled: bool) {
        if enabled {
            self.dispcnt |= DisplayControl::WIN1;
        } else {
            self.dispcnt &= !DisplayControl::WIN1;
        }
    }

    pub fn get_window1_left(&self) -> u16 {
        self.win1_h & 0xFF
    }

    pub fn set_window1_left(&mut self, val: u16) {
        self.win1_h = (self.win1_h & !0xFF) | (val & 0xFF);
    }

    pub fn get_window1_right(&self) -> u16 {
        (self.win1_h >> 8) & 0xFF
    }

    pub fn set_window1_right(&mut self, val: u16) {
        self.win1_h = (self.win1_h & !0xFF00) | ((val & 0xFF) << 8);
    }

    pub fn get_window1_top(&self) -> u16 {
        self.win1_v & 0xFF
    }

    pub fn set_window1_top(&mut self, val: u16) {
        self.win1_v = (self.win1_v & !0xFF) | (val & 0xFF);
    }

    pub fn get_window1_bottom(&self) -> u16 {
        (self.win1_v >> 8) & 0xFF
    }

    pub fn set_window1_bottom(&mut self, val: u16) {
        self.win1_v = (self.win1_v & !0xFF00) | ((val & 0xFF) << 8);
    }

    pub fn is_window1_bg_enabled(&self, bg: usize) -> bool {
        if bg > 3 {
            return false;
        }
        self.winin & (1 << bg) != 0
    }

    pub fn set_window1_bg_enable(&mut self, bg: usize, enabled: bool) {
        if bg > 3 {
            return;
        }
        if enabled {
            self.winin |= 1 << bg;
        } else {
            self.winin &= !(1 << bg);
        }
    }

    pub fn is_window1_obj_enabled(&self) -> bool {
        self.winin & (1 << 4) != 0
    }

    pub fn set_window1_obj_enable(&mut self, enabled: bool) {
        if enabled {
            self.winin |= 1 << 4;
        } else {
            self.winin &= !(1 << 4);
        }
    }

    // Blending
    pub fn is_blending_enabled(&self) -> bool {
        self.bldcnt & (1 << 6) != 0
    }

    pub fn set_blending_enabled(&mut self, enabled: bool) {
        if enabled {
            self.bldcnt |= 1 << 6;
        } else {
            self.bldcnt &= !(1 << 6);
        }
    }

    pub fn is_blend_target_bg(&self, bg: usize) -> bool {
        if bg > 3 {
            return false;
        }
        self.bldcnt & (1 << bg) != 0
    }

    pub fn set_blend_target_bg(&mut self, bg: usize, enabled: bool) {
        if bg > 3 {
            return;
        }
        if enabled {
            self.bldcnt |= 1 << bg;
        } else {
            self.bldcnt &= !(1 << bg);
        }
    }

    pub fn is_blend_target_obj(&self, layer: usize) -> bool {
        if layer > 1 {
            return false;
        }
        self.bldcnt & (1 << (4 + layer)) != 0
    }

    pub fn set_blend_target_obj(&mut self, layer: usize, enabled: bool) {
        if layer > 1 {
            return;
        }
        if enabled {
            self.bldcnt |= 1 << (4 + layer);
        } else {
            self.bldcnt &= !(1 << (4 + layer));
        }
    }

    pub fn get_blend_eva(&self) -> u16 {
        self.bldalpha & 0x1F
    }

    pub fn set_blend_eva(&mut self, val: u16) {
        self.bldalpha = (self.bldalpha & !0x1F) | (val.min(16) & 0x1F);
    }

    pub fn get_blend_evb(&self) -> u16 {
        (self.bldalpha >> 8) & 0x1F
    }

    pub fn set_blend_evb(&mut self, val: u16) {
        self.bldalpha = (self.bldalpha & !0x1F00) | ((val.min(16) & 0x1F) << 8);
    }

    // Mode 3: 16-bit bitmap
    pub fn set_pixel_mode3(&mut self, _x: u16, _y: u16, _color: u16) {
        // In real implementation, writes to VRAM
    }

    pub fn get_pixel_mode3(&self, _x: u16, _y: u16) -> u16 {
        // In real implementation, reads from VRAM
        0
    }

    // Mode 4: 8-bit paletted bitmap
    pub fn set_pixel_mode4(&mut self, _x: u16, _y: u16, _index: u8) {
        // In real implementation, writes to VRAM with page switching
    }

    pub fn get_pixel_mode4(&self, _x: u16, _y: u16) -> u8 {
        // In real implementation, reads from VRAM
        0
    }

    // Sprite handling
    pub fn set_sprite_x(&mut self, _num: usize, _x: u16) {
        // In real implementation, writes to OAM
    }

    pub fn get_sprite_x(&self, _num: usize) -> u16 {
        0
    }

    pub fn set_sprite_y(&mut self, _num: usize, _y: u16) {
        // In real implementation, writes to OAM
    }

    pub fn get_sprite_y(&self, _num: usize) -> u16 {
        0
    }

    pub fn set_sprite_tile(&mut self, _num: usize, _tile: u16) {
        // In real implementation, writes to OAM
    }

    pub fn set_sprite_priority(&mut self, _num: usize, _priority: u16) {
        // In real implementation, writes to OAM
    }

    pub fn set_sprite_palette(&mut self, _num: usize, _palette: u16) {
        // In real implementation, writes to OAM
    }

    pub fn set_sprite_enabled(&mut self, _num: usize, _enabled: bool) {
        // In real implementation, writes to OAM
    }

    pub fn is_sprite_enabled(&self, _num: usize) -> bool {
        false
    }

    /// Step the PPU forward by given number of cycles
    pub fn step(&mut self, cycles: u32) {
        if !self.display_enabled {
            return;
        }

        self.hcounter += cycles;

        // Check for HBlank
        if self.hcounter >= 1232 {
            self.hcounter -= 1232;
            self.vcount += 1;

            // Check for VBlank
            if self.vcount >= 228 {
                self.vcount = 0;
            }
        }
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

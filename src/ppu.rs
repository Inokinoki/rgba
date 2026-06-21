//! GBA Picture Processing Unit (PPU)
//!
//! Handles all graphics rendering including:
//! - Display modes 0-5 (tile modes and bitmap modes)
//! - Background layers (up to 4)
//! - Sprite (OBJ) rendering
//! - Special effects (mosaic, alpha blending, windowing)

use bitflags::bitflags;

bitflags! {
    /// Display control flags (DISPCNT) - GBATEK bit positions
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DisplayControl: u16 {
        const MODE0 = 0 << 0;
        const MODE1 = 1 << 0;
        const MODE2 = 2 << 0;
        const MODE3 = 3 << 0;
        const MODE4 = 4 << 0;
        const MODE5 = 5 << 0;
        const MODE_MASK = 0b111;

        const CGB_MODE = 1 << 3;
        const FRAME_1 = 1 << 4;
        const HBLANK_FREE = 1 << 5;
        const OBJ_VRAM_1D = 1 << 6;
        const FORCED_BLANK = 1 << 7;

        const BG0 = 1 << 8;
        const BG1 = 1 << 9;
        const BG2 = 1 << 10;
        const BG3 = 1 << 11;
        const OBJ = 1 << 12;
        const WIN0 = 1 << 13;
        const WIN1 = 1 << 14;
        const OBJ_WIN = 1 << 15;
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
    pub bg_mosaic: u16,
    pub obj_mosaic: u16,

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

    // VRAM buffer (for testing and simple rendering)
    // In a full implementation, this would be in the Memory system
    vram: Box<[u8; 0x18000]>, // 96KB VRAM

    // Sprite data (simplified OAM storage)
    oam: Box<[u8; 0x400]>, // 1KB OAM
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
            vram: Box::new([0; 0x18000]),
            oam: Box::new([0; 0x400]),
        }
    }

    pub fn reset(&mut self) {
        self.dispcnt = DisplayControl::empty();
        self.display_enabled = false;
        self.dispstat = 0;
        self.vcount = 0;
        self.hcounter = 0;
        self.bgcnt = [0; 4];
        self.bg_hofs = [0; 4];
        self.bg_vofs = [0; 4];
        self.bg_affine = [[0x100, 0, 0, 0x100], [0x100, 0, 0, 0x100]];
        self.bg_mosaic = 0;
        self.obj_mosaic = 0;
        self.win0_h = 0;
        self.win0_v = 0;
        self.win1_h = 0;
        self.win1_v = 0;
        self.winin = 0;
        self.winout = 0;
        self.bldcnt = 0;
        self.bldalpha = 0;
        self.bldy = 0;
        self.vram.fill(0);
        self.oam.fill(0);
    }

    /// Sync VRAM data from Memory system
    /// This must be called before rendering to get the latest VRAM state
    pub fn sync_vram(&mut self, vram_data: &[u8]) {
        let len = self.vram.len().min(vram_data.len());
        self.vram[..len].copy_from_slice(&vram_data[..len]);
    }

    /// Get a reference to VRAM (for reading by GUI)
    pub fn vram(&self) -> &[u8] {
        &self.vram[..]
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

    /// Set the full DISPCNT value from memory
    pub fn set_dispcnt(&mut self, val: u16) {
        self.dispcnt = DisplayControl::from_bits_truncate(val);
    }

    /// Get the full DISPCNT value
    pub fn get_dispcnt(&self) -> u16 {
        self.dispcnt.bits()
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

    pub fn get_hcounter(&self) -> u32 {
        self.hcounter
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

    // Display status (DISPSTAT)
    pub fn get_dispstat(&self) -> u16 {
        let mut stat = self.dispstat;
        // Bit 0: VBlank flag (set when in VBlank)
        if self.is_in_vblank() {
            stat |= 0x0001;
        } else {
            stat &= !0x0001;
        }
        // Bit 1: HBlank flag (set when in HBlank)
        if self.is_in_hblank() {
            stat |= 0x0002;
        } else {
            stat &= !0x0002;
        }
        // Bit 2: VCount match flag (not implemented for now)
        stat
    }

    pub fn set_dispstat(&mut self, val: u16) {
        // Only bits 8-15 are writable (VCount setting)
        // Bits 0-2 are status flags, read-only
        // Bits 3-7 are interrupt enables, writable
        self.dispstat = val;
    }

    // Background control
    pub fn is_bg_enabled(&self, bg: usize) -> bool {
        if bg > 3 {
            return false;
        }
        self.dispcnt
            .contains(DisplayControl::from_bits_truncate(1 << (8 + bg)))
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
        // BG tile base is in 16KB units (bits 2-3 of BGCNT)
        // 0 = 0x0000, 1 = 0x4000, 2 = 0x8000, 3 = 0xC000
        ((self.bgcnt[bg] >> 2) & 0x3) * 0x4000
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
        // BG map base is in 2KB units (bits 8-12 of BGCNT)
        // Each unit is 0x800 bytes
        ((self.bgcnt[bg] >> 8) & 0x1F) * 0x800
    }

    pub fn set_bg_map_base(&mut self, bg: usize, base: u16) {
        if bg > 3 {
            return;
        }
        self.bgcnt[bg] = (self.bgcnt[bg] & !0x1F00) | ((base & 0x1F) << 8);
    }

    /// Get the BG control register value
    pub fn get_bgcnt(&self, bg: usize) -> u16 {
        if bg > 3 {
            return 0;
        }
        self.bgcnt[bg]
    }

    /// Set the BG control register value
    pub fn set_bgcnt(&mut self, bg: usize, val: u16) {
        if bg > 3 {
            return;
        }
        self.bgcnt[bg] = val;
    }

    /// Get background horizontal offset
    pub fn get_bg_hofs(&self, bg: usize) -> u16 {
        if bg > 3 {
            return 0;
        }
        self.bg_hofs[bg]
    }

    /// Set background horizontal offset
    pub fn set_bg_hofs(&mut self, bg: usize, val: u16) {
        if bg > 3 {
            return;
        }
        self.bg_hofs[bg] = val & 0x1FF; // 9 bits (0-511)
    }

    /// Get background vertical offset
    pub fn get_bg_vofs(&self, bg: usize) -> u16 {
        if bg > 3 {
            return 0;
        }
        self.bg_vofs[bg]
    }

    /// Set background vertical offset
    pub fn set_bg_vofs(&mut self, bg: usize, val: u16) {
        if bg > 3 {
            return;
        }
        self.bg_vofs[bg] = val & 0x1FF; // 9 bits (0-511)
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

    pub fn get_bg_affine_b(&self, bg: usize) -> u32 {
        if bg == 2 || bg == 3 {
            self.bg_affine[bg - 2][1]
        } else {
            0
        }
    }

    pub fn set_bg_affine_c(&mut self, bg: usize, val: u32) {
        if bg == 2 || bg == 3 {
            self.bg_affine[bg - 2][2] = val;
        }
    }

    pub fn get_bg_affine_c(&self, bg: usize) -> u32 {
        if bg == 2 || bg == 3 {
            self.bg_affine[bg - 2][2]
        } else {
            0
        }
    }

    pub fn set_bg_affine_d(&mut self, bg: usize, val: u32) {
        if bg == 2 || bg == 3 {
            self.bg_affine[bg - 2][3] = val;
        }
    }

    pub fn get_bg_affine_d(&self, bg: usize) -> u32 {
        if bg == 2 || bg == 3 {
            self.bg_affine[bg - 2][3]
        } else {
            0
        }
    }

    // Mosaic
    pub fn get_bg_mosaic_h(&self) -> u16 {
        (self.bg_mosaic & 0xF) + 1
    }

    /// Get BG mosaic horizontal block size (raw, 0-15)
    pub fn get_bg_mosaic_h_raw(&self) -> u16 {
        self.bg_mosaic & 0xF
    }

    /// Get BG mosaic vertical block size (raw, 0-15)
    pub fn get_bg_mosaic_v_raw(&self) -> u16 {
        (self.bg_mosaic >> 4) & 0xF
    }

    pub fn set_bg_mosaic_h(&mut self, val: u16) {
        self.bg_mosaic = (self.bg_mosaic & !0xF) | (val - 1).min(0xF);
    }

    pub fn get_bg_mosaic_v(&self) -> u16 {
        ((self.bg_mosaic >> 4) & 0xF) + 1
    }

    /// Apply BG mosaic to pixel coordinates
    pub fn apply_bg_mosaic(&self, x: u16, y: u16) -> (u16, u16) {
        let mh = (self.bg_mosaic & 0xF) as u16;
        let mv = ((self.bg_mosaic >> 4) & 0xF) as u16;
        if mh == 0 && mv == 0 {
            return (x, y);
        }
        let bx = if mh > 0 { (x / (mh + 1)) * (mh + 1) } else { x };
        let by = if mv > 0 { (y / (mv + 1)) * (mv + 1) } else { y };
        (bx, by)
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
    pub fn is_window0_enabled(&self) -> bool {
        (self.dispcnt.bits() & (1 << 13)) != 0
    }

    pub fn is_window1_enabled(&self) -> bool {
        (self.dispcnt.bits() & (1 << 14)) != 0
    }

    pub fn is_obj_window_enabled(&self) -> bool {
        (self.dispcnt.bits() & (1 << 15)) != 0
    }

    /// Check if a pixel is inside any window and return the visibility mask
    /// Returns: bitfield of which BGs are visible (bits 0-3) and OBJ (bit 4)
    pub fn get_window_visibility(&self, x: u16, y: u16) -> u16 {
        // If no windows enabled, everything is visible
        let win0_en = (self.dispcnt.bits() & (1 << 13)) != 0;
        let win1_en = (self.dispcnt.bits() & (1 << 14)) != 0;
        let obj_win_en = (self.dispcnt.bits() & (1 << 15)) != 0;

        if !win0_en && !win1_en && !obj_win_en {
            return 0x1F; // All BGs + OBJ visible
        }

        // Check WIN0
        if win0_en {
            let left = (self.win0_h & 0xFF) as u16;
            let right = ((self.win0_h >> 8) & 0xFF) as u16;
            let top = (self.win0_v & 0xFF) as u16;
            let bottom = ((self.win0_v >> 8) & 0xFF) as u16;

            if x >= left && x < right && y >= top && y < bottom {
                // Inside WIN0: use WININ low byte
                return self.winin & 0x1F;
            }
        }

        // Check WIN1
        if win1_en {
            let left = (self.win1_h & 0xFF) as u16;
            let right = ((self.win1_h >> 8) & 0xFF) as u16;
            let top = (self.win1_v & 0xFF) as u16;
            let bottom = ((self.win1_v >> 8) & 0xFF) as u16;

            if x >= left && x < right && y >= top && y < bottom {
                // Inside WIN1: use WININ high byte
                return (self.winin >> 8) & 0x1F;
            }
        }

        // Outside all windows: use WINOUT
        self.winout & 0x1F
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

    pub fn set_window0_h(&mut self, val: u16) {
        self.win0_h = val;
    }

    pub fn set_window0_v(&mut self, val: u16) {
        self.win0_v = val;
    }

    pub fn set_window1_h(&mut self, val: u16) {
        self.win1_h = val;
    }

    pub fn set_window1_v(&mut self, val: u16) {
        self.win1_v = val;
    }

    pub fn set_winin(&mut self, val: u16) {
        self.winin = val;
    }

    pub fn set_winout(&mut self, val: u16) {
        self.winout = val;
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

    /// Set the blend control register (BLDCNT)
    pub fn set_blend_control(&mut self, val: u16) {
        self.bldcnt = val;
    }

    pub fn set_blend_alpha(&mut self, val: u16) {
        self.bldalpha = val;
    }

    pub fn get_blend_control(&self) -> u16 {
        self.bldcnt
    }

    pub fn get_blend_alpha(&self) -> u16 {
        self.bldalpha
    }

    /// Get blend mode (bits 6-7 of BLDCNT):
    /// 0 = none, 1 = alpha, 2 = brightness increase, 3 = brightness decrease
    pub fn get_blend_mode(&self) -> u8 {
        ((self.bldcnt >> 6) & 0x3) as u8
    }

    /// Set the blend brightness register (BLDY)
    pub fn set_blend_brightness(&mut self, val: u16) {
        self.bldy = val;
    }

    pub fn get_blend_brightness(&self) -> u16 {
        self.bldy
    }

    // Mode 3: 16-bit bitmap (240x160)
    pub fn set_pixel_mode3(&mut self, x: u16, y: u16, color: u16) {
        // Mode 3: 240x160, 16-bit color
        // VRAM base: 0x0600_0000
        // Each pixel is 2 bytes
        if x < 240 && y < 160 {
            let offset = ((y as usize * 240 + x as usize) * 2) as usize;
            let bytes = color.to_le_bytes();
            self.vram[offset] = bytes[0];
            self.vram[offset + 1] = bytes[1];
        }
    }

    pub fn get_pixel_mode3(&self, x: u16, y: u16) -> u16 {
        // Mode 3: 240x160, 16-bit color
        if x < 240 && y < 160 {
            let offset = ((y as usize * 240 + x as usize) * 2) as usize;
            u16::from_le_bytes([self.vram[offset], self.vram[offset + 1]])
        } else {
            0
        }
    }

    // Mode 4: 8-bit paletted bitmap (240x160)
    pub fn set_pixel_mode4(&mut self, x: u16, y: u16, index: u8) {
        // Mode 4: 240x160, 8-bit palette index
        // Uses page switching for double buffering
        // Page 0: 0x0600_0000, Page 1: 0x0600_A000
        // Each pixel is 1 byte
        if x < 240 && y < 160 {
            let offset = (y as usize * 240 + x as usize) as usize;
            self.vram[offset] = index;
        }
    }

    pub fn get_pixel_mode4(&self, x: u16, y: u16) -> u8 {
        // Mode 4: 240x160, 8-bit palette index
        if x < 240 && y < 160 {
            let offset = (y as usize * 240 + x as usize) as usize;
            self.vram[offset]
        } else {
            0
        }
    }

    // Sprite/OAM handling
    pub fn sync_oam(&mut self, oam_data: &[u8]) {
        let len = self.oam.len().min(oam_data.len());
        self.oam[..len].copy_from_slice(&oam_data[..len]);
    }

    pub fn oam(&self) -> &[u8] {
        &*self.oam
    }

    /// Get OAM attribute word for sprite (3 words = 6 bytes each)
    fn oam_attr(&self, sprite: usize, attr: usize) -> u16 {
        let offset = sprite * 8 + attr * 2;
        if offset + 1 < self.oam.len() {
            u16::from_le_bytes([self.oam[offset], self.oam[offset + 1]])
        } else {
            0
        }
    }

    /// Get sprite shape (0=square, 1=horizontal, 2=vertical) from attr0 bits 14-15
    pub fn sprite_shape(&self, sprite: usize) -> u16 {
        (self.oam_attr(sprite, 0) >> 14) & 0x3
    }

    /// Get sprite size from attr1 bits 14-15
    pub fn sprite_size(&self, sprite: usize) -> u16 {
        (self.oam_attr(sprite, 1) >> 14) & 0x3
    }

    /// Get sprite dimensions (width, height) based on shape and size
    pub fn sprite_dimensions(&self, sprite: usize) -> (u16, u16) {
        let shape = self.sprite_shape(sprite) as usize;
        let size = self.sprite_size(sprite) as usize;
        // 4x4 table of sprite dimensions (width, height)
        const DIMENSIONS: [[[u16; 2]; 4]; 4] = [
            // size 0              size 1                size 2                size 3
            [[8, 8], [16, 16], [32, 32], [64, 64]], // shape 0 (square)
            [[16, 8], [32, 8], [32, 16], [64, 32]], // shape 1 (horizontal)
            [[8, 16], [8, 32], [16, 32], [32, 64]], // shape 2 (vertical)
            [[8, 8], [16, 16], [32, 32], [64, 64]], // shape 3 (prohibited)
        ];
        let w = DIMENSIONS[shape][size][0];
        let h = DIMENSIONS[shape][size][1];
        (w, h)
    }

    /// Check if sprite is double-sized (attr0 bit 9)
    pub fn sprite_double_size(&self, sprite: usize) -> bool {
        (self.oam_attr(sprite, 0) & 0x0200) != 0
    }

    /// Check if sprite is enabled (attr0 bits 14-15 != 10)
    pub fn sprite_is_enabled(&self, sprite: usize) -> bool {
        let attr0 = self.oam_attr(sprite, 0);
        let mode = (attr0 >> 10) & 0x3;
        if mode == 0b10 {
            return false;
        }
        let rot_scale = (attr0 >> 8) & 1 != 0;
        if !rot_scale && (attr0 & 0x0200) != 0 {
            return false;
        }
        true
    }

    pub fn sprite_y(&self, sprite: usize) -> i32 {
        (self.oam_attr(sprite, 0) & 0xFF) as i32
    }

    /// Get sprite X position (9-bit from attr1 bits 0-8)
    pub fn sprite_x(&self, sprite: usize) -> i32 {
        let x = (self.oam_attr(sprite, 1) & 0x1FF) as i32;
        if x >= 256 {
            x - 512
        } else {
            x
        }
    }

    /// Get sprite tile number (10-bit from attr2 bits 0-9)
    pub fn sprite_tile(&self, sprite: usize) -> u16 {
        self.oam_attr(sprite, 2) & 0x3FF
    }

    /// Get sprite priority (2-bit from attr2 bits 10-11)
    pub fn sprite_priority(&self, sprite: usize) -> u16 {
        (self.oam_attr(sprite, 2) >> 10) & 0x3
    }

    /// Get sprite palette number (4-bit from attr2 bits 12-15, only for 16-color mode)
    pub fn sprite_palette(&self, sprite: usize) -> u16 {
        (self.oam_attr(sprite, 2) >> 12) & 0xF
    }

    /// Check if sprite uses 256-color mode (attr0 bit 13 = 0 for 16-color, 1 for 256-color)
    pub fn sprite_is_256color(&self, sprite: usize) -> bool {
        (self.oam_attr(sprite, 0) & 0x2000) != 0
    }

    /// Check if sprite uses horizontal flip (attr1 bit 12, only for non-affine)
    pub fn sprite_flip_h(&self, sprite: usize) -> bool {
        (self.oam_attr(sprite, 1) & 0x1000) != 0
    }

    /// Check if sprite uses vertical flip (attr1 bit 13, only for non-affine)
    pub fn sprite_flip_v(&self, sprite: usize) -> bool {
        (self.oam_attr(sprite, 1) & 0x2000) != 0
    }

    /// Check if sprite is affine (rotation/scaling) mode (attr0 bit 12)
    /// attr0 bit 12: 0 = non-affine, 1 = affine
    pub fn sprite_is_affine(&self, sprite: usize) -> bool {
        (self.oam_attr(sprite, 0) & 0x0100) != 0
    }

    /// Get affine parameter group index (0-31) from attr0 bits 9-13
    /// Each affine parameter group is 4 halfwords (PA, PB, PC, PD)
    /// 32 groups share space with 128 sprites in OAM
    pub fn sprite_rotation_param(&self, sprite: usize) -> usize {
        let attr1 = self.oam_attr(sprite, 1);
        ((attr1 >> 9) & 0x1F) as usize
    }

    /// Get affine rotation parameter PA (4 halfwords per group, at OAM offset group*16+3)
    pub fn sprite_affine_pa(&self, group: usize) -> i16 {
        let offset = group * 16 + 6;
        if offset + 1 < self.oam.len() {
            i16::from_le_bytes([self.oam[offset], self.oam[offset + 1]])
        } else {
            0x100
        }
    }

    /// Get affine rotation parameter PB
    pub fn sprite_affine_pb(&self, group: usize) -> i16 {
        let offset = group * 16 + 14;
        if offset + 1 < self.oam.len() {
            i16::from_le_bytes([self.oam[offset], self.oam[offset + 1]])
        } else {
            0
        }
    }

    /// Get affine rotation parameter PC
    pub fn sprite_affine_pc(&self, group: usize) -> i16 {
        let offset = group * 16 + 22;
        if offset + 1 < self.oam.len() {
            i16::from_le_bytes([self.oam[offset], self.oam[offset + 1]])
        } else {
            0
        }
    }

    /// Get affine rotation parameter PD
    pub fn sprite_affine_pd(&self, group: usize) -> i16 {
        let offset = group * 16 + 30;
        if offset + 1 < self.oam.len() {
            i16::from_le_bytes([self.oam[offset], self.oam[offset + 1]])
        } else {
            0x100
        }
    }

    /// Check if sprite is a sprite-type window mask (attr0 bits 14-15 == 10)
    pub fn sprite_is_window(&self, sprite: usize) -> bool {
        let mode = (self.oam_attr(sprite, 0) >> 10) & 0x3;
        mode == 0b10
    }

    /// Apply OBJ mosaic to pixel coordinates
    /// Returns the snapped dy value (within the sprite)
    pub fn apply_obj_mosaic(&self, sprite_dy: u16, scanline: u16) -> u16 {
        let mv = ((self.obj_mosaic >> 4) & 0xF) as u16;
        if mv == 0 {
            return sprite_dy;
        }
        let block_h = mv + 1;
        let base_y = (scanline / block_h) * block_h;
        let snapped_dy = base_y % 256;
        snapped_dy
    }

    /// Get OBJ mosaic horizontal snap
    pub fn apply_obj_mosaic_h(&self, sprite_dx: u16) -> u16 {
        let mh = (self.obj_mosaic & 0xF) as u16;
        if mh == 0 {
            return sprite_dx;
        }
        let block_w = mh + 1;
        (sprite_dx / block_w) * block_w
    }

    /// Get a pixel from an OBJ tile
    /// obj_base: 0x10000 (OBJ VRAM starts at offset 0x10000 in VRAM)
    /// tile_num: tile number
    /// x, y: pixel within tile (0-7)
    /// palette_num: palette number (0-15) for 4bpp, ignored for 8bpp
    /// is_256color: true for 256-color mode
    pub fn get_obj_tile_pixel(
        &self,
        tile_num: u16,
        x: u8,
        y: u8,
        _palette_num: u16,
        is_256color: bool,
    ) -> u8 {
        let obj_base = 0x10000; // OBJ tiles start at VRAM offset 0x10000
        if is_256color {
            // 8bpp: tile_num already accounts for 2x size via caller's *2 multiplier
            let tile_offset = obj_base + (tile_num as usize * 32);
            let pixel_offset = tile_offset + (y as usize * 8) + (x as usize);
            if pixel_offset < self.vram.len() {
                self.vram[pixel_offset]
            } else {
                0
            }
        } else {
            // 4bpp: each tile is 32 bytes
            let tile_offset = obj_base + (tile_num as usize * 32);
            let row_offset = tile_offset + (y as usize * 4);
            let byte_offset = row_offset + (x as usize / 2);
            if byte_offset < self.vram.len() {
                let byte = self.vram[byte_offset];
                if x % 2 == 0 {
                    byte & 0x0F
                } else {
                    (byte >> 4) & 0x0F
                }
            } else {
                0
            }
        }
    }

    // === Tile Mode Rendering ===

    /// Read a 16-bit value from VRAM at the given offset
    fn read_vram_half(&self, offset: usize) -> u16 {
        if offset + 1 < self.vram.len() {
            u16::from_le_bytes([self.vram[offset], self.vram[offset + 1]])
        } else {
            0
        }
    }

    /// Get palette color (RGB555) for the given palette index
    /// pal_num: 0 for BG palette, 1 for OBJ palette
    /// index: color index (0-255)
    pub fn get_palette_color(&self, _pal_num: usize, _index: u16) -> u16 {
        // Palette is stored in Memory, not PPU
        // For now, we'll need to get this from Memory
        // This is a placeholder - the actual implementation will be in Gba
        0
    }

    /// Get tile pixel for 4bpp tile (mode 0, 1, 2 text backgrounds)
    /// tile_base: VRAM offset to tile data (character base)
    /// tile_num: tile number
    /// x, y: pixel within tile (0-7)
    /// palette_num: palette number (0-15) for 4bpp tiles
    /// flip_h: horizontal flip
    /// flip_v: vertical flip
    pub fn get_tile_pixel_4bpp(
        &self,
        tile_base: usize,
        tile_num: u16,
        x: u8,
        y: u8,
        _palette_num: u16,
        flip_h: bool,
        flip_v: bool,
    ) -> u8 {
        // Each 4bpp tile is 32 bytes (8x8 pixels, 4 bits per pixel)
        let tile_offset = tile_base + (tile_num as usize * 32);

        // Handle flipping
        let x = if flip_h { 7 - x } else { x };
        let y = if flip_v { 7 - y } else { y };

        // Each row is 4 bytes (8 pixels at 4 bits each)
        let row_offset = tile_offset + (y as usize * 4);

        // Each pixel is 4 bits (nibble)
        let pixel_nibble = if x % 2 == 0 {
            // Low nibble
            (self.vram[row_offset + (x as usize / 2)]) & 0x0F
        } else {
            // High nibble
            (self.vram[row_offset + (x as usize / 2)]) >> 4
        };

        pixel_nibble
    }

    /// Get tile pixel for 8bpp tile (mode 4 bitmap, or mode 2/4 BG with 256-color)
    /// tile_base: VRAM offset to tile data
    /// tile_num: tile number
    /// x, y: pixel within tile (0-7)
    /// flip_h: horizontal flip
    /// flip_v: vertical flip
    pub fn get_tile_pixel_8bpp(
        &self,
        tile_base: usize,
        tile_num: u16,
        x: u8,
        y: u8,
        flip_h: bool,
        flip_v: bool,
    ) -> u8 {
        // Each 8bpp tile is 64 bytes (8x8 pixels, 8 bits per pixel)
        let tile_offset = tile_base + (tile_num as usize * 64);

        // Handle flipping
        let x = if flip_h { 7 - x } else { x };
        let y = if flip_v { 7 - y } else { y };

        // Each row is 8 bytes
        let pixel_offset = tile_offset + (y as usize * 8) + (x as usize);

        self.vram[pixel_offset]
    }

    /// Get screen entry (tile map entry) for text backgrounds
    /// screen_base: VRAM offset to screen block (map base)
    /// x, y: tile position in screen (varies by BG size)
    /// bg_size: background size (0-3 from BG Control register)
    pub fn get_screen_entry(
        &self,
        screen_base: usize,
        x: u16,
        y: u16,
        _bg_size: u16,
        width: u16,
        height: u16,
    ) -> u16 {
        let screen_x = x % width;
        let screen_y = y % height;
        let num_blocks_x = width / 32;
        let block_x = (screen_x / 32) as usize;
        let block_y = (screen_y / 32) as usize;
        let local_x = (screen_x % 32) as usize;
        let local_y = (screen_y % 32) as usize;
        let block_num = block_y * (num_blocks_x as usize) + block_x;
        let entry_offset = screen_base + block_num * 0x800 + (local_y * 32 + local_x) * 2;

        self.read_vram_half(entry_offset)
    }

    /// Parse screen entry to get tile information
    pub fn parse_screen_entry(entry: u16) -> (u16, bool, bool, u16, u16) {
        let tile_num = entry & 0x3FF;
        let flip_h = (entry & 0x400) != 0;
        let flip_v = (entry & 0x800) != 0;
        let palette_num = (entry >> 12) & 0xF;
        let priority = (entry >> 10) & 0x3;

        (tile_num, flip_h, flip_v, palette_num, priority)
    }

    /// Step the PPU forward by given number of cycles
    pub fn step(&mut self, cycles: u32) {
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

    /// Step the PPU and return (vblank_started, hblank_started)
    /// VBlank starts at scanline 160 (when vcount transitions from 159 to 160)
    /// HBlank starts when hcounter crosses 960 (visible pixels end)
    pub fn step_vblank_check(&mut self, cycles: u32) -> (bool, bool) {
        let old_vcount = self.vcount;
        let old_hblank = self.is_in_hblank();

        self.hcounter += cycles;

        let mut vblank_start = false;

        if self.hcounter >= 1232 {
            self.hcounter -= 1232;
            self.vcount += 1;

            // Check for VBlank
            if self.vcount >= 228 {
                self.vcount = 0;
            }

            // VBlank starts when we transition from scanline 159 to 160
            if old_vcount == 159 && self.vcount == 160 {
                vblank_start = true;
            }
        }

        // HBlank starts when hcounter crosses 960 (visible pixels end)
        let new_hblank = self.is_in_hblank();
        let hblank_start = !old_hblank && new_hblank;

        (vblank_start, hblank_start)
    }
}

/// Snapshot of PPU state for parallel rendering
/// Contains all data needed to render a frame without accessing the original PPU
#[derive(Clone)]
pub struct PpuSnapshot {
    pub vram: Box<[u8; 0x18000]>,
    pub oam: Box<[u8; 0x400]>,
    pub dispcnt: u16,
    pub bgcnt: [u16; 4],
    pub bg_hofs: [u16; 4],
    pub bg_vofs: [u16; 4],
    pub bldcnt: u16,
    pub bldalpha: u16,
    pub bldy: u16,
    pub bg_mosaic: u16,
    pub obj_mosaic: u16,
    pub vcount: u16,
}

impl Ppu {
    /// Create a snapshot of current PPU state for parallel rendering
    pub fn snapshot(&self) -> PpuSnapshot {
        PpuSnapshot {
            vram: self.vram.clone(),
            oam: self.oam.clone(),
            dispcnt: self.dispcnt.bits(),
            bgcnt: self.bgcnt,
            bg_hofs: self.bg_hofs,
            bg_vofs: self.bg_vofs,
            bldcnt: self.bldcnt,
            bldalpha: self.bldalpha,
            bldy: self.bldy,
            bg_mosaic: self.bg_mosaic,
            obj_mosaic: self.obj_mosaic,
            vcount: self.vcount,
        }
    }

    /// Render a scanline from a snapshot (can be called from a separate thread)
    pub fn render_scanline_from_snapshot(
        snapshot: &PpuSnapshot,
        scanline: u16,
        framebuffer: &mut [u32],
        palette: &[u8; 0x400],
    ) {
        if scanline >= 160 {
            return;
        }

        let mode = (snapshot.dispcnt & 0x7) as u8;
        let width = 240usize;
        let y = scanline as usize;

        // Collect 15-bit colors first, then batch convert to 32-bit ARGB
        let mut colors_15bit = [0u16; 240];

        for x in 0..width {
            colors_15bit[x] = match mode {
                0 | 1 | 2 => {
                    // Tile modes - render all BG layers and composite
                    Self::render_tile_pixel_composited(snapshot, x as u16, y as u16, palette)
                }
                3 => {
                    // Mode 3: 16-bit bitmap (240x160)
                    let offset = (y * 240 + x) * 2;
                    if offset + 1 < snapshot.vram.len() {
                        u16::from_le_bytes([snapshot.vram[offset], snapshot.vram[offset + 1]])
                    } else {
                        0
                    }
                }
                4 => {
                    // Mode 4: 8-bit paletted bitmap (240x160, double buffered)
                    let page = if snapshot.dispcnt & (1 << 4) != 0 {
                        0xA000
                    } else {
                        0
                    };
                    let offset = page + y * 240 + x;
                    if offset < snapshot.vram.len() {
                        let idx = snapshot.vram[offset] as usize;
                        let pal_offset = idx * 2;
                        if pal_offset + 1 < palette.len() {
                            u16::from_le_bytes([palette[pal_offset], palette[pal_offset + 1]])
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                }
                5 => {
                    // Mode 5: 16-bit bitmap (160x128, double buffered)
                    let page = if snapshot.dispcnt & (1 << 4) != 0 {
                        0xA000
                    } else {
                        0
                    };
                    if x < 160 && y < 128 {
                        let offset = page + (y * 160 + x) * 2;
                        if offset + 1 < snapshot.vram.len() {
                            u16::from_le_bytes([snapshot.vram[offset], snapshot.vram[offset + 1]])
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                }
                _ => 0,
            };
        }

        // Batch convert 15-bit to 32-bit ARGB
        Self::convert_colors_15bit_to_argb(&colors_15bit, framebuffer);
    }

    /// Render a pixel from tile mode with multi-layer compositing
    fn render_tile_pixel_composited(
        snapshot: &PpuSnapshot,
        x: u16,
        y: u16,
        palette: &[u8; 0x400],
    ) -> u16 {
        // Collect enabled backgrounds with their priorities
        let mut bg_layers: [(u8, u16); 4] = [(0, 0); 4]; // (bg_idx, priority)
        let mut layer_count = 0;

        for bg in 0..4 {
            if snapshot.dispcnt & (1 << (8 + bg)) != 0 {
                let priority = snapshot.bgcnt[bg] & 0x3;
                bg_layers[layer_count] = (bg as u8, priority);
                layer_count += 1;
            }
        }

        // Sort by priority (lower number = higher priority)
        bg_layers[..layer_count].sort_by_key(|&(_, pri)| pri);

        // Render each layer and composite
        let mut bg_color = 0u16;
        let mut bg_priority = 4u16; // Start with lowest priority

        for &(bg_idx, priority) in &bg_layers[..layer_count] {
            let color = Self::render_bg_pixel(snapshot, bg_idx as usize, x, y, palette);
            if color != 0 {
                bg_color = color;
                bg_priority = priority;
                break; // Found non-transparent pixel
            }
        }

        // Render sprites if OBJ is enabled
        if snapshot.dispcnt & (1 << 12) != 0 {
            let sprite_color = Self::render_sprite_pixel(snapshot, x, y, palette, bg_priority);
            if sprite_color != 0 {
                return sprite_color;
            }
        }

        // Return background color or backdrop
        if bg_color != 0 {
            bg_color
        } else {
            // All layers transparent, return backdrop color (palette[0])
            if palette.len() >= 2 {
                u16::from_le_bytes([palette[0], palette[1]])
            } else {
                0
            }
        }
    }

    /// Render a sprite pixel at the given position
    fn render_sprite_pixel(
        snapshot: &PpuSnapshot,
        x: u16,
        y: u16,
        palette: &[u8; 0x400],
        max_priority: u16,
    ) -> u16 {
        let obj_tile_base = 0x10000;

        for sprite in 0..128u16 {
            let attr0 = Self::oam_attr_from_data(&snapshot.oam, sprite as usize, 0);
            let obj_mode = (attr0 >> 10) & 0x3;
            if obj_mode == 0b10 {
                continue;
            }

            let rot_scale = (attr0 >> 8) & 1 != 0;
            if !rot_scale && (attr0 & 0x0200) != 0 {
                continue;
            }

            let attr2 = Self::oam_attr_from_data(&snapshot.oam, sprite as usize, 2);
            let sprite_priority = (attr2 >> 10) & 0x3;

            if sprite_priority > max_priority {
                continue;
            }

            let attr1 = Self::oam_attr_from_data(&snapshot.oam, sprite as usize, 1);
            let sy = Self::sprite_y_from_attr(attr0);
            let sx = Self::sprite_x_from_attr(attr1);

            let shape = (attr0 >> 14) & 0x3;
            let size = (attr1 >> 14) & 0x3;
            let (width, height) = Self::sprite_dimensions_from_shape_size(shape, size);

            let dx = x as i32 - sx;
            let dy = y as i32 - sy;

            if dx < 0 || dx >= width as i32 || dy < 0 || dy >= height as i32 {
                continue;
            }

            let tile_num = attr2 & 0x3FF;
            let palette_num = (attr2 >> 12) & 0xF;
            let is_256color = (attr0 & 0x2000) != 0;

            let flip_h = (attr1 & 0x1000) != 0;
            let flip_v = (attr1 & 0x2000) != 0;

            let mut px = dx as u16;
            let mut py = dy as u16;

            if flip_h {
                px = width - 1 - px;
            }
            if flip_v {
                py = height - 1 - py;
            }

            // Calculate tile number based on position
            let tile_x = px / 8;
            let tile_y = py / 8;
            let pixel_x = (px % 8) as u8;
            let pixel_y = (py % 8) as u8;

            // For 256-color sprites, tiles are laid out differently
            let actual_tile = if is_256color {
                // 256-color: tiles are 8x8 but stored linearly
                let tiles_per_row = width / 8;
                tile_num + tile_y * tiles_per_row + tile_x
            } else {
                // 16-color: tiles are stored in 2D blocks
                tile_num + tile_y * (width / 8) + tile_x
            };

            // Get pixel color
            let color_idx = if is_256color {
                // 8bpp
                let tile_offset = obj_tile_base + (actual_tile as usize * 64);
                let pixel_offset = tile_offset + (pixel_y as usize * 8) + (pixel_x as usize);
                if pixel_offset < snapshot.vram.len() {
                    snapshot.vram[pixel_offset] as usize
                } else {
                    0
                }
            } else {
                // 4bpp
                let tile_offset = obj_tile_base + (actual_tile as usize * 32);
                let row_offset = tile_offset + (pixel_y as usize * 4);
                let nibble = if pixel_x % 2 == 0 {
                    if row_offset + (pixel_x as usize / 2) < snapshot.vram.len() {
                        snapshot.vram[row_offset + (pixel_x as usize / 2)] & 0x0F
                    } else {
                        0
                    }
                } else {
                    if row_offset + (pixel_x as usize / 2) < snapshot.vram.len() {
                        snapshot.vram[row_offset + (pixel_x as usize / 2)] >> 4
                    } else {
                        0
                    }
                };
                if nibble == 0 {
                    continue; // Transparent
                }
                (palette_num as usize * 16 + nibble as usize) & 0xFF
            };

            // Look up color in OBJ palette (starts at 0x200 in palette RAM)
            let pal_offset = if is_256color {
                0x200 + color_idx * 2
            } else {
                (0x200 + color_idx * 2) & 0x3FF
            };

            if pal_offset + 1 < palette.len() {
                let color = u16::from_le_bytes([palette[pal_offset], palette[pal_offset + 1]]);
                if color != 0 {
                    return color;
                }
            }
        }

        0
    }

    /// Helper to read OAM attribute
    fn oam_attr_from_data(oam: &[u8; 0x400], sprite: usize, attr: usize) -> u16 {
        let offset = sprite * 8 + attr * 2;
        if offset + 1 < oam.len() {
            u16::from_le_bytes([oam[offset], oam[offset + 1]])
        } else {
            0
        }
    }

    /// Get sprite Y position from attr0
    fn sprite_y_from_attr(attr0: u16) -> i32 {
        let y = (attr0 & 0xFF) as i32;
        if y >= 160 {
            y - 256
        } else {
            y
        }
    }

    /// Get sprite X position from attr1
    fn sprite_x_from_attr(attr1: u16) -> i32 {
        let x = (attr1 & 0x1FF) as i32;
        if x >= 240 {
            x - 512
        } else {
            x
        }
    }

    /// Get sprite dimensions from shape and size
    fn sprite_dimensions_from_shape_size(shape: u16, size: u16) -> (u16, u16) {
        const DIMENSIONS: [[[u16; 2]; 4]; 4] = [
            [[8, 8], [16, 16], [32, 32], [64, 64]], // shape 0 (square)
            [[16, 8], [32, 8], [32, 16], [64, 32]], // shape 1 (horizontal)
            [[8, 16], [8, 32], [16, 32], [32, 64]], // shape 2 (vertical)
            [[8, 8], [16, 16], [32, 32], [64, 64]], // shape 3 (prohibited)
        ];
        let w = DIMENSIONS[shape as usize][size as usize][0];
        let h = DIMENSIONS[shape as usize][size as usize][1];
        (w, h)
    }

    /// Render a pixel from a specific background layer
    fn render_bg_pixel(
        snapshot: &PpuSnapshot,
        bg_idx: usize,
        x: u16,
        y: u16,
        palette: &[u8; 0x400],
    ) -> u16 {
        let bgcnt = snapshot.bgcnt[bg_idx];
        let hofs = snapshot.bg_hofs[bg_idx];
        let vofs = snapshot.bg_vofs[bg_idx];

        // Calculate tile map dimensions based on BG size
        let bg_size = bgcnt & 0x3;
        let (map_width, map_height) = match bg_size {
            0 => (256, 256), // 32x32 tiles
            1 => (512, 256), // 64x32 tiles
            2 => (256, 512), // 32x64 tiles
            3 => (512, 512), // 64x64 tiles
            _ => (256, 256),
        };

        // Calculate pixel position with scrolling
        let px = (x.wrapping_add(hofs)) % map_width;
        let py = (y.wrapping_add(vofs)) % map_height;

        // Calculate tile position
        let tile_x = px / 8;
        let tile_y = py / 8;
        let pixel_in_tile_x = (px % 8) as u8;
        let pixel_in_tile_y = (py % 8) as u8;

        // Get tile base and map base from BGCNT
        let char_base = ((bgcnt >> 2) & 0x3) as usize * 0x4000;
        let screen_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;

        let tiles_per_row = map_width / 8;
        let num_blocks_x = (tiles_per_row / 32) as usize;
        let block_x = (tile_x / 32) as usize;
        let block_y = (tile_y / 32) as usize;
        let local_x = (tile_x % 32) as usize;
        let local_y = (tile_y % 32) as usize;
        let block_num = block_y * num_blocks_x + block_x;
        let entry_offset = screen_base + block_num * 0x800 + (local_y * 32 + local_x) * 2;

        if entry_offset + 1 >= snapshot.vram.len() {
            return 0;
        }
        let entry =
            u16::from_le_bytes([snapshot.vram[entry_offset], snapshot.vram[entry_offset + 1]]);

        let tile_num = entry & 0x3FF;
        let flip_h = (entry & 0x400) != 0;
        let flip_v = (entry & 0x800) != 0;
        let palette_num = (entry >> 12) & 0xF;

        let is_8bpp = (bgcnt >> 7) & 1 != 0;

        let color_idx = if is_8bpp {
            // 8bpp: 256 colors, no palette selection
            let tile_offset = char_base + (tile_num as usize * 64);
            let fx = if flip_h {
                7 - pixel_in_tile_x
            } else {
                pixel_in_tile_x
            };
            let fy = if flip_v {
                7 - pixel_in_tile_y
            } else {
                pixel_in_tile_y
            };
            let pixel_offset = tile_offset + (fy as usize * 8) + (fx as usize);
            if pixel_offset < snapshot.vram.len() {
                snapshot.vram[pixel_offset] as usize
            } else {
                0
            }
        } else {
            // 4bpp: 16 colors per palette
            let tile_offset = char_base + (tile_num as usize * 32);
            let fx = if flip_h {
                7 - pixel_in_tile_x
            } else {
                pixel_in_tile_x
            };
            let fy = if flip_v {
                7 - pixel_in_tile_y
            } else {
                pixel_in_tile_y
            };
            let row_offset = tile_offset + (fy as usize * 4);
            let nibble = if fx % 2 == 0 {
                if row_offset + (fx as usize / 2) < snapshot.vram.len() {
                    snapshot.vram[row_offset + (fx as usize / 2)] & 0x0F
                } else {
                    0
                }
            } else {
                if row_offset + (fx as usize / 2) < snapshot.vram.len() {
                    snapshot.vram[row_offset + (fx as usize / 2)] >> 4
                } else {
                    0
                }
            };
            if nibble == 0 {
                return 0; // Transparent
            }
            (palette_num as usize * 16 + nibble as usize) & 0xFF
        };

        // Look up color in palette
        let pal_offset = color_idx * 2;
        if pal_offset + 1 < palette.len() {
            u16::from_le_bytes([palette[pal_offset], palette[pal_offset + 1]])
        } else {
            0
        }
    }

    /// Convert 15-bit RGB colors to 32-bit ARGB using SIMD when available
    #[inline(always)]
    fn convert_colors_15bit_to_argb(colors: &[u16; 240], framebuffer: &mut [u32]) {
        #[cfg(target_arch = "aarch64")]
        {
            unsafe { Self::convert_colors_15bit_to_argb_neon(colors, framebuffer) }
        }

        #[cfg(not(target_arch = "aarch64"))]
        {
            Self::convert_colors_15bit_to_argb_scalar(colors, framebuffer)
        }
    }

    /// Scalar fallback for color conversion
    #[inline(always)]
    fn convert_colors_15bit_to_argb_scalar(colors: &[u16; 240], framebuffer: &mut [u32]) {
        for i in 0..240 {
            let color = colors[i] as u32;
            let r = ((color & 0x1F) * 255 / 31) << 16;
            let g = (((color >> 5) & 0x1F) * 255 / 31) << 8;
            let b = ((color >> 10) & 0x1F) * 255 / 31;
            framebuffer[i] = r | g | b;
        }
    }

    /// NEON-optimized color conversion for aarch64
    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    unsafe fn convert_colors_15bit_to_argb_neon(colors: &[u16; 240], framebuffer: &mut [u32]) {
        use std::arch::aarch64::*;

        // Process 8 pixels at a time (NEON can do 8x u16 -> 8x u32)
        let mut i = 0;
        while i + 8 <= 240 {
            // Load 8 u16 colors
            let colors_ptr = colors.as_ptr().add(i);
            let c16 = vld1q_u16(colors_ptr);

            // Extract R, G, B components (5 bits each)
            let r16 = vandq_u16(c16, vdupq_n_u16(0x1F));
            let g16 = vandq_u16(vshrq_n_u16(c16, 5), vdupq_n_u16(0x1F));
            let b16 = vandq_u16(vshrq_n_u16(c16, 10), vdupq_n_u16(0x1F));

            // Widen to u32 and scale 0-31 to 0-255
            // (x * 255 + 15) / 31 ≈ x * 8.226, but we use (x << 3) | (x >> 2) for speed
            let r32_lo = vmovl_u16(vget_low_u16(r16));
            let r32_hi = vmovl_u16(vget_high_u16(r16));
            let g32_lo = vmovl_u16(vget_low_u16(g16));
            let g32_hi = vmovl_u16(vget_high_u16(g16));
            let b32_lo = vmovl_u16(vget_low_u16(b16));
            let b32_hi = vmovl_u16(vget_high_u16(b16));

            // Scale: (x << 3) | (x >> 2) gives 0-255 range
            let scale =
                |x: uint32x4_t| -> uint32x4_t { vorrq_u32(vshlq_n_u32(x, 3), vshrq_n_u32(x, 2)) };

            let r_scaled_lo = scale(r32_lo);
            let r_scaled_hi = scale(r32_hi);
            let g_scaled_lo = scale(g32_lo);
            let g_scaled_hi = scale(g32_hi);
            let b_scaled_lo = scale(b32_lo);
            let b_scaled_hi = scale(b32_hi);

            // Shift and combine: (R << 16) | (G << 8) | B
            let argb_lo = vorrq_u32(
                vorrq_u32(vshlq_n_u32(r_scaled_lo, 16), vshlq_n_u32(g_scaled_lo, 8)),
                b_scaled_lo,
            );
            let argb_hi = vorrq_u32(
                vorrq_u32(vshlq_n_u32(r_scaled_hi, 16), vshlq_n_u32(g_scaled_hi, 8)),
                b_scaled_hi,
            );

            // Store results
            let fb_ptr = framebuffer.as_mut_ptr().add(i);
            vst1q_u32(fb_ptr, argb_lo);
            vst1q_u32(fb_ptr.add(4), argb_hi);

            i += 8;
        }

        // Handle remaining pixels with scalar
        while i < 240 {
            let color = colors[i] as u32;
            let r = ((color & 0x1F) * 255 / 31) << 16;
            let g = (((color >> 5) & 0x1F) * 255 / 31) << 8;
            let b = ((color >> 10) & 0x1F) * 255 / 31;
            framebuffer[i] = r | g | b;
            i += 1;
        }
    }

    fn render_tile_pixel_from_snapshot(
        snapshot: &PpuSnapshot,
        x: u16,
        y: u16,
        palette: &[u8; 0x400],
    ) -> u16 {
        // Render BG0 (simplified - just render the first enabled background)
        let mode = (snapshot.dispcnt & 0x7) as u8;

        // Find the first enabled background
        let bg_idx = if snapshot.dispcnt & (1 << 8) != 0 {
            0
        } else if snapshot.dispcnt & (1 << 9) != 0 {
            1
        } else if snapshot.dispcnt & (1 << 10) != 0 {
            2
        } else if snapshot.dispcnt & (1 << 11) != 0 {
            3
        } else {
            return 0;
        }; // No background enabled

        let bgcnt = snapshot.bgcnt[bg_idx];
        let hofs = snapshot.bg_hofs[bg_idx];
        let vofs = snapshot.bg_vofs[bg_idx];

        // Calculate tile map dimensions based on BG size
        let bg_size = bgcnt & 0x3;
        let (map_width, map_height) = match bg_size {
            0 => (256, 256), // 32x32 tiles
            1 => (512, 256), // 64x32 tiles
            2 => (256, 512), // 32x64 tiles
            3 => (512, 512), // 64x64 tiles
            _ => (256, 256),
        };

        // Calculate pixel position with scrolling
        let px = (x.wrapping_add(hofs)) % map_width;
        let py = (y.wrapping_add(vofs)) % map_height;

        // Calculate tile position
        let tile_x = px / 8;
        let tile_y = py / 8;
        let pixel_in_tile_x = (px % 8) as u8;
        let pixel_in_tile_y = (py % 8) as u8;

        // Get tile base and map base from BGCNT
        let char_base = ((bgcnt >> 2) & 0x3) as usize * 0x4000;
        let screen_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;

        let tiles_per_row = map_width / 8;
        let num_blocks_x = (tiles_per_row / 32) as usize;
        let block_x = (tile_x / 32) as usize;
        let block_y = (tile_y / 32) as usize;
        let local_x = (tile_x % 32) as usize;
        let local_y = (tile_y % 32) as usize;
        let block_num = block_y * num_blocks_x + block_x;
        let entry_offset = screen_base + block_num * 0x800 + (local_y * 32 + local_x) * 2;

        if entry_offset + 1 >= snapshot.vram.len() {
            return 0;
        }
        let entry =
            u16::from_le_bytes([snapshot.vram[entry_offset], snapshot.vram[entry_offset + 1]]);

        let tile_num = entry & 0x3FF;
        let flip_h = (entry & 0x400) != 0;
        let flip_v = (entry & 0x800) != 0;
        let palette_num = (entry >> 12) & 0xF;

        let is_8bpp = (bgcnt >> 7) & 1 != 0;

        let color_idx = if is_8bpp {
            // 8bpp: 256 colors, no palette selection
            let tile_offset = char_base + (tile_num as usize * 64);
            let fx = if flip_h {
                7 - pixel_in_tile_x
            } else {
                pixel_in_tile_x
            };
            let fy = if flip_v {
                7 - pixel_in_tile_y
            } else {
                pixel_in_tile_y
            };
            let pixel_offset = tile_offset + (fy as usize * 8) + (fx as usize);
            if pixel_offset < snapshot.vram.len() {
                snapshot.vram[pixel_offset] as usize
            } else {
                0
            }
        } else {
            // 4bpp: 16 colors per palette
            let tile_offset = char_base + (tile_num as usize * 32);
            let fx = if flip_h {
                7 - pixel_in_tile_x
            } else {
                pixel_in_tile_x
            };
            let fy = if flip_v {
                7 - pixel_in_tile_y
            } else {
                pixel_in_tile_y
            };
            let row_offset = tile_offset + (fy as usize * 4);
            let nibble = if fx % 2 == 0 {
                if row_offset + (fx as usize / 2) < snapshot.vram.len() {
                    snapshot.vram[row_offset + (fx as usize / 2)] & 0x0F
                } else {
                    0
                }
            } else {
                if row_offset + (fx as usize / 2) < snapshot.vram.len() {
                    snapshot.vram[row_offset + (fx as usize / 2)] >> 4
                } else {
                    0
                }
            };
            if nibble == 0 {
                return 0; // Transparent
            }
            (palette_num as usize * 16 + nibble as usize) & 0xFF
        };

        // Look up color in palette
        let pal_offset = color_idx * 2;
        if pal_offset + 1 < palette.len() {
            u16::from_le_bytes([palette[pal_offset], palette[pal_offset + 1]])
        } else {
            0
        }
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

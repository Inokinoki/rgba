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

    // VRAM buffer (for testing and simple rendering)
    // In a full implementation, this would be in the Memory system
    vram: Box<[u8; 0x18000]>, // 96KB VRAM

    // Sprite data (simplified OAM storage)
    sprites: [(u16, u16, u16, u16, bool); 128], // (x, y, tile, priority, enabled)
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
            sprites: [(0, 0, 0, 0, false); 128],
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
        self.sprites = [(0, 0, 0, 0, false); 128];
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

    /// Set the blend control register (BLDCNT)
    pub fn set_blend_control(&mut self, val: u16) {
        self.bldcnt = val;
    }

    /// Set the blend alpha register (BLDALPHA)
    pub fn set_blend_alpha(&mut self, val: u16) {
        self.bldalpha = val;
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

    // Sprite handling
    pub fn set_sprite_x(&mut self, num: usize, x: u16) {
        if num < 128 {
            self.sprites[num].0 = x;
        }
    }

    pub fn get_sprite_x(&self, num: usize) -> u16 {
        if num < 128 {
            self.sprites[num].0
        } else {
            0
        }
    }

    pub fn set_sprite_y(&mut self, num: usize, y: u16) {
        if num < 128 {
            self.sprites[num].1 = y;
        }
    }

    pub fn get_sprite_y(&self, num: usize) -> u16 {
        if num < 128 {
            self.sprites[num].1
        } else {
            0
        }
    }

    pub fn set_sprite_tile(&mut self, num: usize, tile: u16) {
        if num < 128 {
            self.sprites[num].2 = tile;
        }
    }

    pub fn set_sprite_priority(&mut self, num: usize, priority: u16) {
        if num < 128 {
            self.sprites[num].3 = priority;
        }
    }

    pub fn set_sprite_palette(&mut self, _num: usize, _palette: u16) {
        // Stored in OAM in real implementation
    }

    pub fn set_sprite_enabled(&mut self, num: usize, enabled: bool) {
        if num < 128 {
            self.sprites[num].4 = enabled;
        }
    }

    pub fn is_sprite_enabled(&self, num: usize) -> bool {
        if num < 128 {
            self.sprites[num].4
        } else {
            false
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
    pub fn get_palette_color(&self, pal_num: usize, index: u16) -> u16 {
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
        palette_num: u16,
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
        bg_size: u16,
        width: u16,
        height: u16,
    ) -> u16 {
        // Screen entry size varies by BG:
        // BG size 0: 256x256 (32x32 tiles) - 1 screen block
        // BG size 1: 512x256 (64x32 tiles) - 2 screen blocks
        // BG size 2: 256x512 (32x64 tiles) - 2 screen blocks
        // BG size 3: 512x512 (64x64 tiles) - 4 screen blocks

        let screen_x = x % width;
        let screen_y = y % height;
        let entry_offset = screen_base + ((screen_y * width + screen_x) as usize * 2);

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

    /// Step the PPU and return true if VBlank just started
    /// VBlank starts at scanline 160 (when vcount transitions from 159 to 160)
    pub fn step_vblank_check(&mut self, cycles: u32) -> bool {
        let old_vcount = self.vcount;

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

        // VBlank starts when we transition from scanline 159 to 160
        old_vcount == 159 && self.vcount == 160
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

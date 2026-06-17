//! Behavior Driven Development tests for the GBA PPU (Picture Processing Unit)
//!
//! These tests describe the expected behavior of the GBA's graphics system,
//! including different display modes, backgrounds, sprites, and effects.

use rgba::Ppu;

/// Scenario: PPU initializes with correct display settings
#[test]
fn ppu_initializes_with_correct_default_state() {
    let ppu = Ppu::new();

    // Then: Display should be off initially
    assert_eq!(ppu.is_display_enabled(), false, "Display should start disabled");

    // And: Should be in a valid display mode
    assert_eq!(ppu.get_display_mode(), 0, "Should start in mode 0");

    // And: VBlank and HBlank counters should be at 0
    assert_eq!(ppu.get_vcount(), 0, "VCOUNT should start at 0");
    assert_eq!(ppu.is_in_vblank(), false, "Should not be in VBlank");
    assert_eq!(ppu.is_in_hblank(), false, "Should not be in HBlank");
}

/// Scenario: PPU can enable and disable display
#[test]
fn ppu_can_enable_and_disable_display() {
    let mut ppu = Ppu::new();

    // Given: Display is disabled
    assert_eq!(ppu.is_display_enabled(), false);

    // When: Display is enabled
    ppu.set_display_enabled(true);

    // Then: Display should be enabled
    assert_eq!(ppu.is_display_enabled(), true);

    // When: Display is disabled
    ppu.set_display_enabled(false);

    // Then: Display should be disabled
    assert_eq!(ppu.is_display_enabled(), false);
}

/// Scenario: PPU supports different display modes
#[test]
fn ppu_supports_all_display_modes() {
    let mut ppu = Ppu::new();

    // Modes 0-2: Tile/text modes
    for mode in 0..=2 {
        ppu.set_display_mode(mode);
        assert_eq!(ppu.get_display_mode(), mode, "Should be in mode {}", mode);
    }

    // Mode 3: 240x160 16-bit bitmap
    ppu.set_display_mode(3);
    assert_eq!(ppu.get_display_mode(), 3);
    assert_eq!(ppu.get_width(), 240, "Mode 3 width should be 240");
    assert_eq!(ppu.get_height(), 160, "Mode 3 height should be 160");

    // Mode 4: 240x160 8-bit bitmap with page switching
    ppu.set_display_mode(4);
    assert_eq!(ppu.get_display_mode(), 4);
    assert_eq!(ppu.get_width(), 240, "Mode 4 width should be 240");
    assert_eq!(ppu.get_height(), 160, "Mode 4 height should be 160");

    // Mode 5: 160x128 16-bit bitmap with page switching
    ppu.set_display_mode(5);
    assert_eq!(ppu.get_display_mode(), 5);
    assert_eq!(ppu.get_width(), 160, "Mode 5 width should be 160");
    assert_eq!(ppu.get_height(), 128, "Mode 5 height should be 128");
}

/// Scenario: PPU correctly generates VBlank signal
#[test]
fn ppu_generates_vblank_signal_correctly() {
    let mut ppu = Ppu::new();
    ppu.set_display_enabled(true);

    // GBA: 160 visible scanlines, then 68 VBlank lines
    // Total: 228 scanlines per frame

    // When: Scanning through visible lines
    for line in 0..160 {
        ppu.set_vcount(line);
        assert_eq!(ppu.is_in_vblank(), false, "Line {} should not be VBlank", line);
    }

    // Then: VBlank should start at line 160
    for line in 160..228 {
        ppu.set_vcount(line);
        assert_eq!(ppu.is_in_vblank(), true, "Line {} should be VBlank", line);
    }

    // And: VBlank should end at line 228 (wraps to 0)
    ppu.set_vcount(0);
    assert_eq!(ppu.is_in_vblank(), false, "Line 0 should not be VBlank");
}

/// Scenario: PPU correctly generates HBlank signal
#[test]
fn ppu_generates_hblank_signal_correctly() {
    let mut ppu = Ppu::new();
    ppu.set_display_enabled(true);

    // Each scanline: 240 visible pixels, then HBlank
    // Total: 1232 cycles per scanline (960 visible + 272 HBlank at 16.78MHz)

    // HBlank timing is handled during stepping
    // When: Starting a new scanline
    ppu.set_vcount(10);
    ppu.set_hcounter(0);

    // Then: Should not be in HBlank
    assert_eq!(ppu.is_in_hblank(), false);

    // When: Reaching HBlank period
    ppu.set_hcounter(960); // End of visible pixels

    // Then: Should be in HBlank
    assert_eq!(ppu.is_in_hblank(), true);
}

/// Scenario: PPU renders mode 3 (16-bit bitmap) correctly
#[test]
fn ppu_renders_mode_3_bitmap_correctly() {
    let mut ppu = Ppu::new();
    ppu.set_display_mode(3);
    ppu.set_display_enabled(true);

    // Given: VRAM with pixel data
    // Mode 3: Direct color, 240x160 = 38,400 pixels = 76,800 bytes

    // When: Setting a pixel at (x, y)
    let x = 120;
    let y = 80;
    let color = 0x7FFF; // White

    ppu.set_pixel_mode3(x, y, color);

    // Then: Should read back correctly
    assert_eq!(ppu.get_pixel_mode3(x, y), color, "Pixel color should match");

    // And: Adjacent pixels should be independent
    ppu.set_pixel_mode3(x + 1, y, 0x001F); // Red
    assert_eq!(ppu.get_pixel_mode3(x + 1, y), 0x001F);
    assert_eq!(ppu.get_pixel_mode3(x, y), 0x7FFF); // Unchanged
}

/// Scenario: PPU renders mode 4 (8-bit paletted bitmap) correctly
#[test]
fn ppu_renders_mode_4_paletted_bitmap_correctly() {
    let mut ppu = Ppu::new();
    ppu.set_display_mode(4);
    ppu.set_display_enabled(true);

    // Mode 4: 8-bit palette index, 240x160 = 38,400 pixels = 38,400 bytes
    // Double buffered: 0x0600_0000 and 0x0600_A000

    // When: Setting a pixel with palette index
    let x = 100;
    let y = 50;
    let palette_index = 123;

    ppu.set_pixel_mode4(x, y, palette_index);

    // Then: Should read back correctly
    assert_eq!(ppu.get_pixel_mode4(x, y), palette_index);
}

/// Scenario: PPU renders tile-based backgrounds (modes 0-2)
#[test]
fn ppu_renders_tile_backgrounds_correctly() {
    let mut ppu = Ppu::new();
    ppu.set_display_mode(0); // Text mode, all 4 BG layers available
    ppu.set_display_enabled(true);

    // When: Enabling a background layer
    ppu.set_bg_enabled(0, true);

    // Then: BG should be enabled
    assert_eq!(ppu.is_bg_enabled(0), true);

    // When: Setting background control properties
    ppu.set_bg_priority(0, 1);
    ppu.set_bg_tile_base(0, 0); // Character base block 0
    ppu.set_bg_map_base(0, 31); // Screen base block 31

    // Then: Properties should be stored
    assert_eq!(ppu.get_bg_priority(0), 1);
}

/// Scenario: PPU handles affine backgrounds (mode 2)
#[test]
fn ppu_handles_affine_backgrounds() {
    let mut ppu = Ppu::new();
    ppu.set_display_mode(2); // Mode 2: BG2 and BG3 are affine
    ppu.set_display_enabled(true);

    // Affine backgrounds use transformation matrices
    let bg = 2; // BG2 is first affine

    // When: Setting transformation parameters
    ppu.set_bg_affine_a(bg, 0x100); // P.A (16.16 fixed point)
    ppu.set_bg_affine_b(bg, 0x000); // P.B
    ppu.set_bg_affine_c(bg, 0x000); // P.C
    ppu.set_bg_affine_d(bg, 0x100); // P.D

    // Then: Parameters should affect rendering
    assert_eq!(ppu.get_bg_affine_a(bg), 0x100);
}

/// Scenario: PPU renders sprites (OBJ) correctly
#[test]
fn ppu_renders_sprites_correctly() {
    let mut ppu = Ppu::new();
    ppu.set_display_enabled(true);

    // OAM can have up to 128 sprites
    // When: Setting up a sprite
    let sprite_num = 0;
    ppu.set_sprite_x(sprite_num, 120);
    ppu.set_sprite_y(sprite_num, 80);
    ppu.set_sprite_tile(sprite_num, 5);
    ppu.set_sprite_priority(sprite_num, 2);
    ppu.set_sprite_palette(sprite_num, 0);

    // Then: Sprite should be configured
    assert_eq!(ppu.get_sprite_x(sprite_num), 120);
    assert_eq!(ppu.get_sprite_y(sprite_num), 80);

    // When: Enabling sprite
    ppu.set_sprite_enabled(sprite_num, true);

    // Then: Sprite should be enabled
    assert_eq!(ppu.is_sprite_enabled(sprite_num), true);
}

/// Scenario: PPU handles mosaic effect
#[test]
fn ppu_handles_mosaic_effect() {
    let mut ppu = Ppu::new();
    ppu.set_display_enabled(true);

    // Mosaic creates pixelated blocks
    // When: Setting mosaic parameters
    ppu.set_bg_mosaic_h(4); // 4 pixel horizontal block
    ppu.set_bg_mosaic_v(4); // 4 pixel vertical block
    ppu.set_obj_mosaic_h(2);
    ppu.set_obj_mosaic_v(2);

    // Then: Mosaic should be applied during rendering
    assert_eq!(ppu.get_bg_mosaic_h(), 4);
    assert_eq!(ppu.get_bg_mosaic_v(), 4);
}

/// Scenario: PPU handles window effects
#[test]
fn ppu_handles_window_effects() {
    let mut ppu = Ppu::new();
    ppu.set_display_enabled(true);

    // Windows allow selective display of layers
    // When: Setting up window 1
    ppu.set_window1_enabled(true);
    ppu.set_window1_left(10);
    ppu.set_window1_right(230);
    ppu.set_window1_top(5);
    ppu.set_window1_bottom(155);

    // Then: Window should be configured
    assert_eq!(ppu.is_window1_enabled(), true);
    assert_eq!(ppu.get_window1_left(), 10);

    // When: Setting which layers appear in window
    ppu.set_window1_bg_enable(0, true);
    ppu.set_window1_bg_enable(1, false);
    ppu.set_window1_obj_enable(true);

    // Then: Layer control should be set
    assert_eq!(ppu.is_window1_bg_enabled(0), true);
}

/// Scenario: PPU handles alpha blending
#[test]
fn ppu_handles_alpha_blending() {
    let mut ppu = Ppu::new();
    ppu.set_display_enabled(true);

    // BLDCTRL controls blending
    // When: Enabling alpha blending
    ppu.set_blending_enabled(true);

    // Then: Blending should be enabled
    assert_eq!(ppu.is_blending_enabled(), true);

    // When: Setting blend targets
    ppu.set_blend_target_bg(0, true); // BG0 is top layer
    ppu.set_blend_target_bg(1, true); // BG1 is bottom layer
    ppu.set_blend_target_obj(0, true); // OBJ is top layer

    // Then: Targets should be configured
    assert_eq!(ppu.is_blend_target_bg(0), true);

    // When: Setting blend weights
    ppu.set_blend_eva(8); // Top layer weight (0-16)
    ppu.set_blend_evb(8); // Bottom layer weight (0-16)

    // Then: Weights should be stored
    assert_eq!(ppu.get_blend_eva(), 8);
    assert_eq!(ppu.get_blend_evb(), 8);
}

/// Scenario: PPU stepping updates scanlines correctly
#[test]
fn ppu_stepping_advances_scanlines() {
    let mut ppu = Ppu::new();
    ppu.set_display_enabled(true);

    // Each scanline takes 1232 cycles
    let cycles_per_line = 1232;

    // When: Stepping for one line
    ppu.step(cycles_per_line);

    // Then: VCOUNT should advance
    assert_eq!(ppu.get_vcount(), 1);

    // When: Stepping through visible area
    for _ in 0..159 {
        ppu.step(cycles_per_line);
    }

    // Then: Should be at line 160 (start of VBlank)
    assert_eq!(ppu.get_vcount(), 160);
    assert_eq!(ppu.is_in_vblank(), true);
}

/// Scenario: PPU frame buffer is accessible
#[test]
fn ppu_frame_buffer_is_accessible() {
    let mut ppu = Ppu::new();
    ppu.set_display_mode(3);
    ppu.set_display_enabled(true);

    // When: Rendering a frame
    for y in 0..160 {
        for x in 0..240 {
            let color = ((x as u16) << 5) | (y as u16); // Simple gradient
            ppu.set_pixel_mode3(x, y, color & 0x7FFF);
        }
    }

    // Then: Frame buffer should contain correct data
    let pixel = ppu.get_pixel_mode3(100, 50);
    assert!(pixel != 0);
}

/// Scenario: PPU resets to clean state
#[test]
fn ppu_reset_clears_all_state() {
    let mut ppu = Ppu::new();

    // Given: PPU in some state
    ppu.set_display_enabled(true);
    ppu.set_display_mode(4);
    ppu.set_bg_enabled(0, true);

    // When: Reset
    ppu.reset();

    // Then: Should be back to default state
    assert_eq!(ppu.is_display_enabled(), false);
    assert_eq!(ppu.get_display_mode(), 0);
    assert_eq!(ppu.is_bg_enabled(0), false);
}

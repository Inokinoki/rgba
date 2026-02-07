//! Graphics Demo Example
//!
//! This example demonstrates the PPU (Picture Processing Unit) capabilities:
//! - Setting display modes
//! - Drawing pixels in different modes
//! - Background layer control
//! - Sprite manipulation

use rgba::Gba;

fn main() {
    let mut gba = Gba::new();

    println!("🎨 RGBA GBA Emulator - Graphics Demo");
    println!("====================================");
    println!();

    // Enable display
    gba.ppu.set_display_enabled(true);
    println!("Display enabled");

    // Mode 3: 240x160 16-bit bitmap
    println!();
    println!("Mode 3: 240x160 16-bit bitmap");
    println!("-------------------------------");
    gba.ppu.set_display_mode(3);
    gba.ppu.set_bg_enabled(0, true);

    // Draw a gradient pattern
    println!("Drawing gradient pattern...");
    for y in 0..160 {
        for x in 0..240 {
            // Create a simple RGB555 color gradient
            let r = (x * 31 / 240) as u16;
            let g = (y * 31 / 160) as u16;
            let b = 15;
            let color = (b << 10) | (g << 5) | r;
            gba.ppu.set_pixel_mode3(x, y, color);
        }
    }
    println!("  Drew {} pixels", 240 * 160);

    // Test pixel reading
    let test_pixel = gba.ppu.get_pixel_mode3(120, 80);
    println!("  Pixel at (120, 80): 0x{:04X}", test_pixel);

    // Mode 4: 240x160 8-bit paletted
    println!();
    println!("Mode 4: 240x160 8-bit paletted");
    println!("--------------------------------");
    gba.ppu.set_display_mode(4);

    // Draw pattern using palette indices
    println!("Drawing palette pattern...");
    for y in 0..160 {
        for x in 0..240 {
            let index = ((x / 16) + (y / 16) * 15) % 256;
            gba.ppu.set_pixel_mode4(x, y, index as u8);
        }
    }
    println!("  Drew {} palette indices", 240 * 160);

    // Test palette reading
    let test_index = gba.ppu.get_pixel_mode4(120, 80);
    println!("  Palette index at (120, 80): {}", test_index);

    // Mode 5: 160x128 16-bit bitmap
    println!();
    println!("Mode 5: 160x128 16-bit bitmap");
    println!("--------------------------------");
    gba.ppu.set_display_mode(5);

    // Draw a test pattern in smaller resolution
    for y in 0..128 {
        for x in 0..160 {
            let color = if x < 80 && y < 64 {
                0x001F // Red
            } else if x >= 80 && y < 64 {
                0x03E0 // Green
            } else if x < 80 && y >= 64 {
                0x7C00 // Blue
            } else {
                0x7FFF // White
            };
            gba.ppu.set_pixel_mode3(x, y, color);
        }
    }
    println!("  Drew 4 colored quadrants");

    // Background layer control (for tile modes)
    println!();
    println!("Background Layer Control");
    println!("------------------------");
    gba.ppu.set_display_mode(0);

    // Enable and configure backgrounds
    for bg in 0..4 {
        gba.ppu.set_bg_enabled(bg, true);
        gba.ppu.set_bg_priority(bg, bg as u16);
        println!("  BG{}: enabled, priority {}", bg, bg);
    }

    // Sprite system
    println!();
    println!("Sprite System");
    println!("-------------");

    // Configure a few sprites
    for i in 0..8 {
        gba.ppu.set_sprite_enabled(i, true);
        gba.ppu.set_sprite_x(i, 50 + (i * 20) as u16);
        gba.ppu.set_sprite_y(i, 80);
        gba.ppu.set_sprite_tile(i, i as u16);
        gba.ppu.set_sprite_priority(i, 2);
        println!("  Sprite {}: x={}, y={}, tile={}", i, 50 + i * 20, 80, i);
    }

    // Display timing information
    println!();
    println!("Display Timing");
    println!("--------------");
    println!("  Resolution: {}x{}", gba.ppu.get_width(), gba.ppu.get_height());
    println!("  VCOUNT: {}", gba.ppu.get_vcount());
    println!("  In VBlank: {}", gba.ppu.is_in_vblank());
    println!("  In HBlank: {}", gba.ppu.is_in_hblank());

    println!();
    println!("✅ Graphics demo completed!");
}

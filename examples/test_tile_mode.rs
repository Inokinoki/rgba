//! Test tile mode rendering
//!
//! This example verifies that tile mode rendering methods are available and functional.

use rgba::{Gba, Ppu};

fn main() {
    let gba = Gba::new();

    println!("Tile Mode Rendering Test");
    println!("=======================");

    let ppu = gba.ppu();
    println!("Display mode: {}", ppu.get_display_mode());
    println!("BG0 enabled: {}", ppu.is_bg_enabled(0));

    // Set up a simple tile mode configuration
    println!("\n=== Tile Rendering Methods ===");
    println!("Available methods:");
    println!("  - Ppu::get_tile_pixel_4bpp()");
    println!("  - Ppu::get_tile_pixel_8bpp()");
    println!("  - Ppu::get_screen_entry()");
    println!("  - Ppu::parse_screen_entry()");
    println!("  - Gba::get_pixel_tile_mode()");
    println!("  - Gba::get_palette_color()");

    // Test palette color reading
    println!("\n=== Palette Testing ===");
    let color = gba.get_palette_color(0, 0);
    println!("Palette[0][0] = 0x{:04X} (should be 0x0000 = black/transparent)", color);

    // Test tile pixel reading (empty VRAM should return 0)
    let tile_base = 0;
    println!("\n=== Tile Pixel Testing ===");
    let pixel_4bpp = ppu.get_tile_pixel_4bpp(tile_base, 0, 0, 0, 0, false, false);
    let pixel_8bpp = ppu.get_tile_pixel_8bpp(tile_base, 0, 0, 0, false, false);
    println!("4bpp tile[0] at (0,0): color_index = {}", pixel_4bpp);
    println!("8bpp tile[0] at (0,0): color_index = {}", pixel_8bpp);

    // Test screen entry parsing
    println!("\n=== Screen Entry Testing ===");
    let test_entry = 0x1234;
    let (tile_num, flip_h, flip_v, palette_num, priority) = Ppu::parse_screen_entry(test_entry);
    println!("Test screen entry: 0x{:04X}", test_entry);
    println!("  Tile number: {}", tile_num);
    println!("  Flip H: {}", flip_h);
    println!("  Flip V: {}", flip_v);
    println!("  Palette: {}", palette_num);
    println!("  Priority: {}", priority);

    println!("\n=== GUI Support ===");
    println!("The GUI now supports rendering tile/text modes 0, 1, and 2!");
    println!("Run with: cargo run --example gui_emulator --features gui -- <rom_file>");
    println!("\nControls:");
    println!("  Arrow Keys: D-Pad");
    println!("  Z: A, X: B, Enter: Start, Shift: Select");
    println!("  A: L, S: R, R: Reset, Escape: Quit");
}

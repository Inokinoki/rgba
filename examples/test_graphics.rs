//! Direct graphics test - sets up tile mode graphics manually
//!
//! This bypasses ROM loading and directly sets up VRAM, palette, and registers.

use rgba::Gba;

fn main() {
    let mut gba = Gba::new();

    println!("=== Direct Graphics Test ===");
    println!("Setting up Mode 0 tile graphics directly...\n");

    // === Step 1: Set up palette ===
    // Color format: RGB555 (bit 0-4 = red, 5-9 = green, 10-14 = blue)
    // Set up 16 colors in palette 0
    println!("Setting up palette...");

    // Color 1: White (0x7FFF)
    gba.write_half(0x0500_0002, 0x7FFF);
    // Color 2: Red (0x001F in RGB555 - red is bits 0-4)
    gba.write_half(0x0500_0004, 0x001F);
    // Color 3: Green (0x03E0 - green is bits 5-9)
    gba.write_half(0x0500_0006, 0x03E0);
    // Color 4: Blue (0x7C00 - blue is bits 10-14)
    gba.write_half(0x0500_0008, 0x7C00);
    // Color 5: Yellow (Red + Green = 0x03FF)
    gba.write_half(0x0500_000A, 0x03FF);
    // Color 6: Cyan (Green + Blue = 0x7FE0)
    gba.write_half(0x0500_000C, 0x7FE0);
    // Color 7: Magenta (Red + Blue = 0x7C1F)
    gba.write_half(0x0500_000E, 0x7C1F);

    println!("✓ Palette configured with 7 colors\n");

    // === Step 2: Set up tile data ===
    // Create 8x8 tile patterns in VRAM
    // 4bpp = 4 bits per pixel = 2 pixels per byte
    // Tile 0: A simple checkerboard pattern
    println!("Setting up tile data...");

    let tile_0_data: [u8; 32] = [
        0x11, 0x11, 0x22, 0x22,  // Row 0: 1,1,2,2,1,1,2,2
        0x11, 0x11, 0x22, 0x22,  // Row 1
        0x33, 0x33, 0x44, 0x44,  // Row 2: 3,3,4,4,3,3,4,4
        0x33, 0x33, 0x44, 0x44,  // Row 3
        0x55, 0x55, 0x66, 0x66,  // Row 4: 5,5,6,6,5,5,6,6
        0x55, 0x55, 0x66, 0x66,  // Row 5
        0x77, 0x77, 0x00, 0x00,  // Row 6: 7,7,0,0,7,7,0,0
        0x77, 0x77, 0x00, 0x00,  // Row 7
    ];

    // Write tile 0 to VRAM at 0x0600_0000
    for (i, &byte) in tile_0_data.iter().enumerate() {
        gba.write_byte(0x0600_0000 + i as u32, byte);
    }

    // Tile 1: A solid red tile
    let tile_1_data: [u8; 32] = [0x22; 32]; // All color index 2
    for (i, &byte) in tile_1_data.iter().enumerate() {
        gba.write_byte(0x0600_0020 + i as u32, byte); // Offset 32 = next tile
    }

    println!("✓ 2 tiles written to VRAM\n");

    // === Step 3: Set up screen entry (tile map) ===
    // Map BG0 (256x256 = 32x32 tiles) to display our tiles
    // Screen base for BG0 is at block 24 = 0x0600_8000
    println!("Setting up tile map...");

    // Create a checkerboard pattern of tiles on screen
    for y in 0..32 {
        for x in 0..32 {
            let tile_num = if (x + y) % 2 == 0 { 0 } else { 1 };
            let palette_num = 0u16;
            let entry = tile_num | (palette_num << 12); // Tile number + palette

            let offset = 0x0600_8000 + ((y * 32 + x) * 2) as u32;
            gba.write_half(offset, entry);
        }
    }

    println!("✓ Tile map configured (32x32 tiles)\n");

    // === Step 4: Configure BG0 through IO registers ===
    println!("Configuring BG0 registers...");

    // DISPCNT (0x0400_0000): Enable display, mode 0, BG0 enable
    let dispcnt: u16 = 0x0080 |  // Display enable (bit 7)
                        0x0100 |  // BG0 enable (bit 8)
                        0x0000;   // Mode 0 (bits 0-2)
    gba.write_half(0x0400_0000, dispcnt);

    // BG0CNT (0x0400_0008): Configure BG0
    // Bits: priority(2) | char_base(2) | mosaic(1) | 8bpp(1) | map_base(5) | padding(1) | size(2)
    let bg0cnt: u16 = (0 << 0) |   // Priority 0 (highest)
                      (0 << 2) |   // Character base = block 0 (0x0600_0000)
                      (0 << 6) |   // No mosaic
                      (0 << 7) |   // 4bpp (16 colors)
                      (24 << 8) |  // Screen base = block 24 (0x0600_8000)
                      (0 << 14);   // Size 256x256
    gba.write_half(0x0400_0008, bg0cnt);

    // BG offsets (set to 0)
    gba.write_half(0x0400_0010, 0); // BG0HOFS
    gba.write_half(0x0400_0012, 0); // BG0VOFS

    println!("✓ BG0 configured\n");
    println!("  DISPCNT: 0x{:04X}", dispcnt);
    println!("  BG0CNT: 0x{:04X}", bg0cnt);

    // === Step 5: Sync and verify ===
    println!("\nSyncing PPU state...");
    gba.sync_ppu();

    let ppu = gba.ppu();
    println!("Display enabled: {}", ppu.is_display_enabled());
    println!("Display mode: {}", ppu.get_display_mode());
    println!("BG0 enabled: {}", ppu.is_bg_enabled(0));
    println!("BG0 BGCNT: 0x{:04X}", ppu.get_bgcnt(0));

    // === Step 6: Test pixel reading ===
    println!("\n=== Testing Pixel Reading ===");
    let mut pixel_count = 0;
    for y in [0u16, 10, 50, 100, 150].iter() {
        for x in [0u16, 20, 100, 200].iter() {
            let color = gba.get_pixel_tile_mode(*x, *y);
            if color != 0 {
                pixel_count += 1;
                let r = (color & 0x1F);
                let g = ((color >> 5) & 0x1F);
                let b = ((color >> 10) & 0x1F);
                println!("  ({:3}, {:3}) = 0x{:04X} (R={:2}, G={:2}, B={:2})",
                         x, y, color, r, g, b);
            }
        }
    }

    if pixel_count == 0 {
        println!("  No pixels rendered! Something is wrong.");
        println!("\n=== Diagnostic Info ===");

        // Check palette
        println!("\nPalette check:");
        for i in 0..8 {
            let color = gba.get_palette_color(0, i);
            println!("  [{}] = 0x{:04X}", i, color);
        }

        // Check VRAM
        println!("\nVRAM check (tile 0):");
        let vram = gba.mem().vram();
        for i in 0..32 {
            print!("{:02X} ", vram[i]);
            if (i + 1) % 8 == 0 {
                println!();
            }
        }

        // Check tile map
        println!("\nTile map check (first 16 entries):");
        for i in 0..16 {
            let addr = 0x0600_8000 + (i * 2);
            let entry = gba.mem().read_half(addr);
            println!("  [{}] @ 0x{:08X} = 0x{:04X}", i, addr, entry);
        }
    } else {
        println!("\n✓ {} pixels rendered successfully!", pixel_count);
        println!("\nYou can now run the GUI to see this:");
        println!("  cargo run --example gui_emulator --features gui");
    }
}

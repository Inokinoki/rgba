//! Diagnostic tool to check display state
//!
//! This loads a ROM and runs it for a bit, then dumps the PPU state.

use rgba::Gba;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <rom_file>", args[0]);
        println!("\nThis will:");
        println!("  1. Load the ROM");
        println!("  2. Run for ~1 frame (2800 steps)");
        println!("  3. Sync PPU state");
        println!("  4. Dump display state");
        return Ok(());
    }

    let rom_path = &args[1];

    // Create GBA and load ROM
    let mut gba = Gba::new();
    match gba.load_rom_path(rom_path) {
        Ok(_) => println!("✓ Loaded ROM: {}", rom_path),
        Err(e) => {
            eprintln!("✗ Failed to load ROM: {}", e);
            return Err(e);
        }
    }

    println!("\n=== Running Emulation ===");
    println!("Running for 1 frame (2800 steps)...");

    for i in 0..2800 {
        gba.step();
        if i == 100 || i == 1000 || i == 2700 {
            println!("Step {}...", i);
        }
    }

    // Sync PPU state
    gba.sync_ppu();

    println!("\n=== Display State ===");
    let ppu = gba.ppu();

    println!("Display enabled: {}", ppu.is_display_enabled());
    println!("Display mode: {}", ppu.get_display_mode());
    println!("VCount: {}", ppu.get_vcount());

    println!("\n=== Background State ===");
    for bg in 0..4 {
        println!("BG{}:", bg);
        println!("  Enabled: {}", ppu.is_bg_enabled(bg));
        println!("  Priority: {}", ppu.get_bg_priority(bg));
        let bgcnt = ppu.get_bgcnt(bg);
        println!("  BGCNT: 0x{:04X}", bgcnt);

        // Decode BGCNT
        let priority = bgcnt & 0x3;
        let tile_base = ((bgcnt >> 2) & 0x3) * 16;
        let mosaic = (bgcnt >> 6) & 0x1;
        let colors_8bpp = (bgcnt >> 7) & 0x1;
        let map_base = ((bgcnt >> 8) & 0x1F) * 2;
        let bg_size = (bgcnt >> 14) & 0x3;

        println!("    Priority: {}", priority);
        println!("    Tile base: 0x{:04X}", tile_base);
        println!("    Mosaic: {}", mosaic);
        println!("    Colors: {}", if colors_8bpp != 0 { "8bpp (256)" } else { "4bpp (16)" });
        println!("    Map base: 0x{:04X}", map_base);
        println!("    Size: {}", bg_size);

        println!("  H-offset: {}", ppu.get_bg_hofs(bg));
        println!("  V-offset: {}", ppu.get_bg_vofs(bg));
    }

    println!("\n=== Palette (first 16 colors) ===");
    for i in 0..16 {
        let color = gba.get_palette_color(0, i);
        if color != 0 {
            let r = (color & 0x1F);
            let g = ((color >> 5) & 0x1F);
            let b = ((color >> 10) & 0x1F);
            println!("[{}] = 0x{:04X} (R={}, G={}, B={})", i, color, r, g, b);
        }
    }
    let palette_nonzero = (0..16).filter(|&i| gba.get_palette_color(0, i) != 0).count();
    println!("Non-zero palette entries: {}/16", palette_nonzero);

    println!("\n=== VRAM Sample (first 256 bytes) ===");
    let vram = ppu.vram();
    let mut nonzero_count = 0;
    for i in 0..256 {
        if vram[i] != 0 {
            nonzero_count += 1;
            if nonzero_count <= 20 {
                print!("{:02X} ", vram[i]);
                if (i + 1) % 16 == 0 {
                    println!();
                }
            }
        }
    }
    if nonzero_count == 0 {
        println!("(all zeros)");
    } else if nonzero_count > 20 {
        println!("... ({} non-zero bytes total)", nonzero_count);
    }

    let vram_nonzero = vram.iter().filter(|&&b| b != 0).count();
    println!("Non-zero VRAM bytes: {}/96KB", vram_nonzero);

    println!("\n=== Test Pixel Read ===");
    // Try to read a few pixels using the tile rendering
    println!("Testing tile mode pixel reading:");
    for y in [0, 10, 50, 100, 150].iter() {
        for x in [0, 20, 100, 200].iter() {
            let color = gba.get_pixel_tile_mode(*x, *y);
            if color != 0 {
                let r = (color & 0x1F);
                let g = ((color >> 5) & 0x1F);
                let b = ((color >> 10) & 0x1F);
                println!("  ({}, {}) = 0x{:04X} (R={}, G={}, B={})", x, y, color, r, g, b);
            }
        }
    }

    println!("\n=== Memory IO Registers (first 64 bytes) ===");
    let io = gba.mem().io();
    for i in (0..64).step_by(4) {
        let reg = 0x0400_0000 + i as u32;
        let val = u16::from_le_bytes([io[i], io[i + 1]]);
        if val != 0 {
            println!("0x{:08X} = 0x{:04X}", reg, val);
        }
    }

    Ok(())
}

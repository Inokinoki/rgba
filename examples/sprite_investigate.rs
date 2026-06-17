use rgba::Gba;
use std::fs;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..500 {
        gba.run_frame_parallel(&mut fb);
    }

    // OBJ palette bank 3 (entries 48-63)
    println!("OBJ Palette bank 3 (entries 48-63):");
    for i in 48..64u16 {
        let color = gba.get_palette_color(1, i);
        let r = color & 0x1F;
        let g = (color >> 5) & 0x1F;
        let b = (color >> 10) & 0x1F;
        if color != 0 {
            println!("  obj_pal[{}] = {:04X} (r={} g={} b={})", i, color, r, g, b);
        } else {
            println!("  obj_pal[{}] = 0000 (black)", i);
        }
    }

    // Tile 848 data in OBJ VRAM (0x10000 + 848*32 = 0x10000 + 0x6800 = 0x16800)
    let vram = gba.ppu().vram();
    let obj_base = 0x10000usize;
    let tile_addr = obj_base + 848 * 32;
    println!("\nTile 848 OBJ data (at 0x{:X}):", tile_addr);
    for row in 0..8 {
        let mut pixels = String::new();
        for byte_idx in 0..4 {
            if tile_addr + row * 4 + byte_idx < vram.len() {
                let byte = vram[tile_addr + row * 4 + byte_idx];
                let lo = byte & 0xF;
                let hi = (byte >> 4) & 0xF;
                pixels.push_str(&format!("{:X}{:X} ", lo, hi));
            }
        }
        println!("  row{}: {}", row, pixels);
    }

    // Check the get_obj_tile_pixel function directly
    let ppu = gba.ppu();
    // Sprite 16: tile=848, pal=3, 4bpp, 16x16
    // At pixel (37-24, 79-79) = (13, 0) inside sprite
    // tile_x = 13/8 = 1, tile_y = 0/8 = 0
    // pixel_x = 13%8 = 5, pixel_y = 0%8 = 0
    // actual_tile = 848 + 0 * (16/8) + 1 = 849
    // So we read tile 849
    println!("\nTile 849 OBJ data (at 0x{:X}):", obj_base + 849 * 32);
    let tile_addr2 = obj_base + 849 * 32;
    for row in 0..8 {
        let mut pixels = String::new();
        for byte_idx in 0..4 {
            if tile_addr2 + row * 4 + byte_idx < vram.len() {
                let byte = vram[tile_addr2 + row * 4 + byte_idx];
                let lo = byte & 0xF;
                let hi = (byte >> 4) & 0xF;
                pixels.push_str(&format!("{:X}{:X} ", lo, hi));
            }
        }
        println!("  row{}: {}", row, pixels);
    }

    // Test the actual function
    let color_idx = ppu.get_obj_tile_pixel(849, 5, 0, 3, false);
    println!(
        "\nget_obj_tile_pixel(849, 5, 0, pal=3, 4bpp) = {}",
        color_idx
    );

    let pal_color = gba.get_palette_color(1, 3 * 16 + color_idx as u16);
    println!(
        "OBJ palette[{}] = {:04X}",
        3 * 16 + color_idx as u16,
        pal_color
    );

    // Also check sprite 17 at (40, 79) - pixel (37-40, 79-79) inside sprite would be negative...
    // Wait, x=40 means the sprite starts at x=40
    // Pixel (37, 79) is at dx = 37 - 40 = -3, which is < 0, so sprite 17 doesn't cover pixel 37
    // Let's check which sprites cover (37, 79)
    // Sprite 16: x=24, y=79, 16x16 → covers x=[24,40), y=[79,95) → (37,79) is inside
    // Sprite 17: x=40, y=79, 16x16 → covers x=[40,56), y=[79,95) → (37,79) is NOT inside

    // So only sprite 16 covers pixel (37,79)
    // dx=13, dy=0, tile_x=1, tile_y=0, pixel_x=5, pixel_y=0
    // actual_tile = 848 + 0*(16/8) + 1 = 849

    // Let me also check what mGBA shows for these sprites
    // mGBA frame 500: no sprites at y=79 x=24 or x=40!
    // mGBA shows sprites 0-5 at y=242 (off screen) and sprites 6-9 at y=124/132
    // So our emulator has extra sprites that mGBA doesn't show

    println!("\n=== Key finding ===");
    println!("Our emulator has sprites 16/17 at y=79 (visible black hole area)");
    println!("mGBA has NO sprites at these positions");
    println!("This means OAM content differs between emulators at frame 500");

    // Let's dump first 20 OAM entries from both
    let oam = ppu.oam();
    println!("\nOur first 20 OAM entries:");
    for i in 0..20 {
        let base = i * 8;
        let a0 = u16::from_le_bytes([oam[base], oam[base + 1]]);
        let a1 = u16::from_le_bytes([oam[base + 2], oam[base + 3]]);
        let a2 = u16::from_le_bytes([oam[base + 4], oam[base + 5]]);
        let y = a0 & 0xFF;
        let x = a1 & 0x1FF;
        println!(
            "  [{}] a0={:04X} a1={:04X} a2={:04X} y={} x={}",
            i, a0, a1, a2, y, x
        );
    }
}

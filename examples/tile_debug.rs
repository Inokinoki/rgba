use rgba::Gba;
use std::fs;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..800u32 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..10 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..220 {
        gba.run_frame_parallel(&mut fb);
    }

    let vram = gba.ppu().vram();

    // Tile 1023 in 4bpp = 32 bytes at offset 1023*32 = 0x7FE0
    println!("Tile 1023 data (at 0x7FE0):");
    let tile_addr = 1023 * 32; // 0x7FE0
    for row in 0..8 {
        let mut pixels = String::new();
        for byte_idx in 0..4 {
            let byte = vram[tile_addr + row * 4 + byte_idx];
            pixels.push_str(&format!("{:02X} ", byte));
        }
        println!("  row{}: {}", row, pixels);
    }

    // Also check the raw bytes
    println!("\nRaw bytes at 0x7FE0:");
    for i in 0..32 {
        print!("{:02X} ", vram[0x7FE0 + i]);
        if (i + 1) % 16 == 0 {
            println!();
        }
    }

    // Check palette entry 240-255 (palette bank 15)
    println!("\nPalette bank 15 (entries 240-255):");
    for i in 240..256u16 {
        let color = gba.get_palette_color(0, i);
        if color != 0 {
            let r = color & 0x1F;
            let g = (color >> 5) & 0x1F;
            let b = (color >> 10) & 0x1F;
            println!("  pal[{}] = {:04X} (r={} g={} b={})", i, color, r, g, b);
        }
    }

    // Check what parse_screen_entry returns for 0xFFFF
    let entry: u16 = 0xFFFF;
    let tile_num = entry & 0x3FF;
    let flip_h = (entry >> 10) & 1 != 0;
    let flip_v = (entry >> 11) & 1 != 0;
    let palette_num = ((entry >> 12) & 0xF) as u8;
    println!(
        "\nparse_screen_entry(0xFFFF): tile={} fh={} fv={} pal={}",
        tile_num, flip_h, flip_v, palette_num
    );

    // Check get_tile_pixel_4bpp for tile 1023
    let ppu = gba.ppu();
    for py in 0..8 {
        for px in 0..8 {
            let color_idx = ppu.get_tile_pixel_4bpp(0, 1023, px, py, 15, true, true);
            print!("{:X}", color_idx);
        }
        print!(" ");
    }
    println!();

    // Also: what does the mGBA dialogue frame look like?
    // mGBA at dialogue: DISPCNT=1F40, BG0CNT=5843 map=C000
    // Our: DISPCNT=1640, BG1CNT=1841 map=C000
    // These are COMPLETELY different game states!
    // The game progressed differently in our emulator vs mGBA

    // Let me verify: does mGBA get the same sequence?
    // The issue might be that our emulator's timing is off enough
    // that the game takes a different path

    // Check tile data at tile 540 (first non-FFFF entry in BG1 map)
    println!("\nTile 540 data:");
    let tile_addr2 = 540 * 32;
    for row in 0..8 {
        let mut pixels = String::new();
        for byte_idx in 0..4 {
            let byte = vram[tile_addr2 + row * 4 + byte_idx];
            for bit in 0..4 {
                let shift = bit * 2;
                let pix = (byte >> shift) & 3;
                pixels.push_str(&format!("{}", pix));
            }
        }
        println!("  row{}: {}", row, pixels.chars().rev().collect::<String>());
    }

    // The key question: why does BG1 at (0,0) return None?
    // Entry = 0xFFFF → tile 1023, pal 15
    // Tile 1023 data → let me check if it's all zeros
    // If so, color_index = 0, and get_bg_pixel returns None

    // That would mean the 0xFFFF entries are "transparent" tiles
    // The game uses them as empty space, and other BG layers fill in
    // Let me check BG2 rendering for the same pixel

    let bg2_pixel = gba.get_bg_pixel(gba.ppu(), 0, 2, 0, 0);
    println!("get_bg_pixel(BG2, 0, 0) = {:?}", bg2_pixel);

    // BG2 entry[0] = 0x0258 tile=600 pal=0
    // Let's check tile 600
    println!("\nTile 600 data:");
    let tile_addr3 = 600 * 32;
    for row in 0..8 {
        let mut pixels = String::new();
        for byte_idx in 0..4 {
            let byte = vram[tile_addr3 + row * 4 + byte_idx];
            for bit in 0..4 {
                let shift = bit * 2;
                let pix = (byte >> shift) & 3;
                pixels.push_str(&format!("{}", pix));
            }
        }
        println!("  row{}: {}", row, pixels.chars().rev().collect::<String>());
    }
}

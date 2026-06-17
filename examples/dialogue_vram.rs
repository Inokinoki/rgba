use rgba::Gba;
use std::fs;

fn save_screenshot(fb: &[u32], png_path: &str) {
    let ppm_path = png_path.replace(".png", ".ppm");
    let mut bytes = b"P6\n240 160\n255\n".to_vec();
    for y in 0..160 {
        for x in 0..240 {
            let pixel = fb[y * 240 + x];
            bytes.push(((pixel >> 16) & 0xFF) as u8);
            bytes.push(((pixel >> 8) & 0xFF) as u8);
            bytes.push((pixel & 0xFF) as u8);
        }
    }
    fs::write(&ppm_path, &bytes).unwrap();
    std::process::Command::new("python3")
        .args([
            "-c",
            &format!(
                "from PIL import Image; Image.open('{}').save('{}')",
                ppm_path, png_path
            ),
        ])
        .output()
        .unwrap();
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Run to dialogue screen
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

    // Advance to after forced blank ends
    for _ in 0..220u32 {
        gba.run_frame_parallel(&mut fb);
    }

    // Now we should be at the dialogue/name-entry screen
    let ppu = gba.ppu();
    let dispcnt = ppu.get_dispcnt();
    println!("DISPCNT={:04X}", dispcnt);

    let bg1cnt = ppu.get_bgcnt(1);
    let bg2cnt = ppu.get_bgcnt(2);
    println!("BG1CNT={:04X}", bg1cnt);
    println!("BG2CNT={:04X}", bg2cnt);

    save_screenshot(&fb, "/tmp/dialogue_fixed.png");

    // Dump BG1 map (0xC000) - full first row (32 entries for 256px wide)
    let vram = ppu.vram();
    println!("\nBG1 map at 0xC000 (entries 0-63):");
    for i in 0..64 {
        let addr = 0xC000 + i * 2;
        if addr + 1 < vram.len() {
            let val = u16::from_le_bytes([vram[addr], vram[addr + 1]]);
            let tile = val & 0x3FF;
            let pal = (val >> 12) & 0xF;
            println!("  [{:2}] {:04X} tile={:4} pal={}", i, val, tile, pal);
        }
    }

    // Dump BG2 map (0xF800) - first row
    println!("\nBG2 map at 0xF800 (entries 0-63):");
    for i in 0..64 {
        let addr = 0xF800 + i * 2;
        if addr + 1 < vram.len() {
            let val = u16::from_le_bytes([vram[addr], vram[addr + 1]]);
            let tile = val & 0x3FF;
            let pal = (val >> 12) & 0xF;
            println!("  [{:2}] {:04X} tile={:4} pal={}", i, val, tile, pal);
        }
    }

    // Check tile data for tiles referenced by BG1 (e.g., 0x01A2 = tile 418)
    // BG1 tile_base = ((bg1cnt >> 2) & 3) * 0x4000 = 0
    println!("\nTile data for tile 418 (at 0x{:X}):", 418 * 32);
    let tile_addr = 418 * 32;
    for row in 0..8 {
        let mut pixels = String::new();
        for byte_idx in 0..4 {
            let byte = vram[tile_addr + row * 4 + byte_idx];
            for bit in 0..4 {
                let shift = bit * 2;
                let pix = (byte >> shift) & 3;
                pixels.push_str(&format!("{}", pix));
            }
        }
        // Reverse for display
        println!("  row{}: {}", row, pixels.chars().rev().collect::<String>());
    }

    // Check palette banks used by BG1 and BG2 entries
    // BG1 uses pal=0 (entries 0x01A2), check pal 0
    println!("\nBG Palette bank 0 (entries 0-15):");
    for i in 0..16u16 {
        let color = gba.get_palette_color(0, i);
        if color != 0 {
            let r = color & 0x1F;
            let g = (color >> 5) & 0x1F;
            let b = (color >> 10) & 0x1F;
            println!("  pal[{}] = {:04X} (r={} g={} b={})", i, color, r, g, b);
        }
    }

    // Check tile 600 (used by BG2 entries 0x0258-0x025C)
    // BG2 tile_base = ((bg2cnt >> 2) & 3) * 0x4000 = 0
    // But BG2CNT = 0x1F02: tile_base bits = (0x1F02 >> 2) & 3 = 0
    // Actually let me recalculate:
    // BG2CNT = 0x1F02: bits [3:2] = 0b00, so tile_base = 0
    // But the 8bpp bit is bit 7: (0x1F02 >> 7) & 1 = 0xF = ... wait
    // BG2CNT = 0001_1111_0000_0010
    // bits [3:2] = 00, tile_base = 0
    // bit 7 = 0 (4bpp)
    // Actually BG2CNT = 0x1F02:
    //   pri = 2
    //   tile_base = ((0x1F02 >> 2) & 3) * 0x4000 = 0
    //   8bpp = (0x1F02 >> 7) & 1 = 0 (4bpp)
    //   map_base = ((0x1F02 >> 8) & 0x1F) * 0x800 = 0x1F * 0x800 = 0xF800
    //   size = (0x1F02 >> 14) & 3 = 0 (256x256)
    // That's correct.

    // Check tile 600
    println!("\nTile data for tile 600 (at 0x{:X}):", 600 * 32);
    let tile_addr2 = 600 * 32;
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

    // Now let's manually render a pixel to check
    // Pixel at (0, 0) - should come from BG1
    // BG1: map_base=0xC000, tile_base=0, size=0 (256x256 text mode)
    // Map entry at 0xC000 = 0x01A2 (tile 418, pal 0)
    // Screen entry row = 0, col = 0
    // Tile row 0, pixel col 0-7

    // Actually let's check: what does get_pixel_tile_mode return for (0,0)?
    let color_00 = gba.get_pixel_tile_mode(0, 0);
    println!("\nget_pixel_tile_mode(0,0) = {:04X}", color_00);

    // Let's also check what BG1 pixel at (0,0) returns
    let bg1_pixel = gba.get_bg_pixel(ppu, 0, 1, 0, 0);
    println!("get_bg_pixel(BG1, 0, 0) = {:?}", bg1_pixel);

    // Check OAM for visible sprites
    let oam = ppu.oam();
    println!("\nFirst 20 OAM entries:");
    for i in 0..20 {
        let base = i * 8;
        let a0 = u16::from_le_bytes([oam[base], oam[base + 1]]);
        let a1 = u16::from_le_bytes([oam[base + 2], oam[base + 3]]);
        let a2 = u16::from_le_bytes([oam[base + 4], oam[base + 5]]);
        let y = a0 & 0xFF;
        let x = a1 & 0x1FF;
        let shape = (a0 >> 14) & 3;
        let size_bits = (a1 >> 14) & 3;
        let tile_num = a2 & 0x3FF;
        let priority = (a2 >> 10) & 3;
        let pal_bank = (a2 >> 12) & 0xF;

        if y < 160 || shape != 0 {
            println!("  [{:2}] y={:3} x={:3} shape={} size={} tile={} pri={} pal={:2} a0={:04X} a1={:04X} a2={:04X}",
                i, y, x, shape, size_bits, tile_num, priority, pal_bank, a0, a1, a2);
        }
    }

    // Compare mGBA's BG1 map at dialogue frame:
    // mGBA: entry[0] = B1D9 (tile=473 pal=11)
    // Our: entry[0] at 0xC000 = 01A2 (tile=418 pal=0)
    // These are COMPLETELY different! The map data is different.
    // This means the game wrote different data to VRAM at 0xC000.

    // Let me check mem.vram() vs ppu.vram() to see if sync is correct
    let mem_vram = gba.mem().vram();
    println!("\nMem VRAM at 0xC000 (first 10 entries):");
    for i in 0..10 {
        let addr = 0xC000 + i * 2;
        let val = u16::from_le_bytes([mem_vram[addr], mem_vram[addr + 1]]);
        print!(" {:04X}", val);
    }
    println!();

    println!("\nPPU VRAM at 0xC000 (first 10 entries):");
    for i in 0..10 {
        let addr = 0xC000 + i * 2;
        let val = u16::from_le_bytes([vram[addr], vram[addr + 1]]);
        print!(" {:04X}", val);
    }
    println!();
}

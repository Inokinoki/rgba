use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    // The BG0 hscroll = 0x00E0 = 224 pixels
    // In a 256-pixel wide tilemap (size=1, 32x32 tiles = 256x256 pixels)
    // hscroll=224 means visible area starts at column 224, wraps to column 0 at pixel 32
    // So we should see tiles at column 224/8=28, 29, 30, 31, 0, 1, 2...

    let bg0cnt = gba.mem.read_half(0x04000008);
    let hscroll = gba.mem.read_half(0x04000010) & 0x1FF;
    println!("BG0CNT={:04X} hscroll={}", bg0cnt, hscroll);
    println!("  Priority: {} (0=highest, 3=lowest)", bg0cnt & 3);
    println!(
        "  Visible tiles at columns: {}, {}, {}, {}, ...",
        hscroll / 8,
        (hscroll / 8 + 1) % 32,
        (hscroll / 8 + 2) % 32,
        (hscroll / 8 + 3) % 32
    );

    // Manually check what pixel the PPU would render at position (0,0) for BG0
    // screen_x=0 -> map_x = (0 + 224) % 256 = 224 -> tile_col = 224/8 = 28
    // screen_y=0 -> map_y = 0 -> tile_row = 0/8 = 0
    let map_x = (0 + hscroll as usize) % 256;
    let map_y = 0usize;
    let tile_col = map_x / 8;
    let tile_row = map_y / 8;
    let tile_base = 0xC000;
    let entry_off = tile_base + (tile_row * 64 + tile_col) * 2;
    let vram = gba.mem.vram();
    let entry = u16::from_le_bytes([vram[entry_off], vram[entry_off + 1]]);
    let tile_idx = (entry & 0x3FF) as usize;
    let pal = ((entry >> 12) & 0xF) as usize;
    println!(
        "\nAt screen (0,0): map=({},{}) tile_col={} tile_row={}",
        map_x, map_y, tile_col, tile_row
    );
    println!(
        "  Tile map entry: 0x{:04X} -> tile={} pal={}",
        entry, tile_idx, pal
    );

    // Check the tile pixel at this position
    let pixel_x = map_x % 8;
    let pixel_y = map_y % 8;
    let byte_off = tile_idx * 32 + pixel_y * 4 + pixel_x / 2;
    let byte = vram[byte_off];
    let color_idx = if pixel_x % 2 == 0 {
        byte & 0xF
    } else {
        (byte >> 4) & 0xF
    };
    println!("  Pixel ({},{}): color_idx={}", pixel_x, pixel_y, color_idx);

    // Now check what pixel (120, 80) would show - middle of screen
    let map_x2 = (120 + hscroll as usize) % 256;
    let map_y2 = 80usize;
    let tile_col2 = map_x2 / 8;
    let tile_row2 = map_y2 / 8;
    let entry_off2 = tile_base + (tile_row2 * 64 + tile_col2) * 2;
    let entry2 = u16::from_le_bytes([vram[entry_off2], vram[entry_off2 + 1]]);
    let tile_idx2 = (entry2 & 0x3FF) as usize;
    let pal2 = ((entry2 >> 12) & 0xF) as usize;
    println!(
        "\nAt screen (120,80): map=({},{}) tile_col={} tile_row={}",
        map_x2, map_y2, tile_col2, tile_row2
    );
    println!(
        "  Tile map entry: 0x{:04X} -> tile={} pal={}",
        entry2, tile_idx2, pal2
    );

    // Check BG3 (pri=0, highest priority) tile at same position
    let bg3tb = 0xF000;
    let entry3_off = bg3tb + (tile_row2 * 64 + tile_col2) * 2;
    let entry3 = u16::from_le_bytes([vram[entry3_off], vram[entry3_off + 1]]);
    let tile3 = entry3 & 0x3FF;
    let pal3 = (entry3 >> 12) & 0xF;
    println!("\nBG3 at same position: tile={} pal={}", tile3, pal3);

    // BG3 tile pixel data
    if tile3 < 1024 {
        let byte3_off = tile3 as usize * 32 + (map_y2 % 8) * 4 + ((map_x2 % 8) / 2);
        let byte3 = vram[byte3_off];
        let c3 = if (map_x2 % 8) % 2 == 0 {
            byte3 & 0xF
        } else {
            (byte3 >> 4) & 0xF
        };
        println!(
            "  BG3 pixel ({},{}): color_idx={}",
            map_x2 % 8,
            map_y2 % 8,
            c3
        );
    }

    // Check actual framebuffer color at (120, 80)
    let fb_color = fb[80 * 240 + 120];
    println!("\nFramebuffer at (120,80): 0x{:06X}", fb_color & 0xFFFFFF);

    // Render just BG0 to a PPM file for inspection
    for row in 0..160u32 {
        for col in 0..240u32 {
            let mx = ((col as usize + hscroll as usize) % 256) as u32;
            let my = row as usize;
            let tc = (mx / 8) as usize;
            let tr = (my / 8) as usize;
            let eo = tile_base + (tr * 64 + tc) * 2;
            let te = u16::from_le_bytes([vram[eo], vram[eo + 1]]);
            let ti = (te & 0x3FF) as usize;
            let pp = (te >> 12) & 0xF;
            let px = (mx % 8) as usize;
            let py = (my % 8) as usize;
            let bo = ti * 32 + py * 4 + px / 2;
            let bv = vram[bo];
            let ci = if px % 2 == 0 {
                bv & 0xF
            } else {
                (bv >> 4) & 0xF
            };
            if ci != 0 {
                fb[(row * 240 + col) as usize] = 0xFF0000 | (ci as u32);
            } else {
                fb[(row * 240 + col) as usize] = 0;
            }
        }
    }
    let mut ppm = String::from("P3\n240 160\n255\n");
    for y in 0..160 {
        for x in 0..240 {
            let c = fb[y * 240 + x];
            let r = (c >> 16) & 0xFF;
            let g = (c >> 8) & 0xFF;
            let b = c & 0xFF;
            ppm.push_str(&format!("{} {} {} ", r, g, b));
        }
        ppm.push('\n');
    }
    std::fs::write("/tmp/bg0_only.ppm", ppm).unwrap();
    println!("\nBG0 only saved to /tmp/bg0_only.ppm");
}

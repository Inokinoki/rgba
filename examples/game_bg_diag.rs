use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for _ in 0..1000 {
        gba.run_frame();
    }

    gba.sync_ppu_full();
    gba.sync_ppu();

    let io = gba.mem().io();
    let dispcnt = u16::from_le_bytes([io[0], io[1]]);
    println!("DISPCNT={:#06X} mode={}", dispcnt, dispcnt & 7);

    for bg in 0..4 {
        let bgcnt_off = 0x08 + bg * 2;
        let bgcnt = u16::from_le_bytes([io[bgcnt_off], io[bgcnt_off + 1]]);
        let char_base = ((bgcnt >> 2) & 0xF) as u32 * 0x4000;
        let screen_base = ((bgcnt >> 8) & 0x1F) as u32 * 0x800;
        let size = (bgcnt >> 14) & 3;
        let priority = bgcnt & 3;
        let mosaic = (bgcnt >> 6) & 1;
        let palette_256 = (bgcnt >> 7) & 1;
        let enabled = (dispcnt >> (8 + bg)) & 1;
        println!("BG{}: CNT={:#06X} pri={} char={:#06X} screen={:#06X} size={} pal256={} mosaic={} enabled={}",
            bg, bgcnt, priority, char_base, screen_base, size, palette_256, mosaic, enabled);

        if enabled != 0 && (dispcnt & 7) == 0 {
            let vram = gba.ppu().vram();
            let screen_bytes = screen_base as usize;

            let mut nonzero_tiles = 0u32;
            let mut total_tiles = 0u32;
            let tile_count = if size == 0 { 32 * 32 } else { 64 * 32 };

            for i in 0..tile_count {
                let entry_off = screen_bytes + i * 2;
                if entry_off + 1 < vram.len() {
                    let entry = u16::from_le_bytes([vram[entry_off], vram[entry_off + 1]]);
                    let tile_num = entry & 0x3FF;
                    total_tiles += 1;
                    if tile_num != 0x3FF {
                        nonzero_tiles += 1;
                    }
                    if i < 32 && tile_num != 0x3FF {
                        println!("  screen_entry[{}]={:#06X} tile={}", i, entry, tile_num);
                    }
                }
            }
            println!("  Non-empty tiles: {}/{}", nonzero_tiles, total_tiles);
        }
    }

    // Check palette
    let pal = gba.mem().palette();
    println!("\n=== Palette (BG) ===");
    for i in 0..16 {
        let off = i * 2;
        let c = u16::from_le_bytes([pal[off], pal[off + 1]]);
        if c != 0 {
            let r = (c & 0x1F) as u32 * 255 / 31;
            let g = ((c >> 5) & 0x1F) as u32 * 255 / 31;
            let b = ((c >> 10) as u32) * 255 / 31;
            println!("  Pal[{}]={:#06X} rgb=({},{},{})", i, c, r, g, b);
        }
    }

    // Check tile data at char_base 0
    let vram = gba.ppu().vram();
    println!("\n=== First few tiles at char_base 0 ===");
    for tile in 0..5 {
        print!("Tile {}: ", tile);
        let base = tile * 32;
        let mut has_data = false;
        for byte in 0..32 {
            if vram[base + byte] != 0 {
                has_data = true;
                break;
            }
        }
        if has_data {
            for byte in 0..32 {
                print!("{:02X}", vram[base + byte]);
            }
            println!();
        } else {
            println!("(empty)");
        }
    }

    // Check tile 1023 (the "empty" tile)
    println!("\n=== Tile 1023 ===");
    let tile1023_off = 1023 * 32;
    if tile1023_off + 32 <= vram.len() {
        let mut all_ff = true;
        for byte in 0..32 {
            if vram[tile1023_off + byte] != 0xFF {
                all_ff = false;
            }
        }
        println!("All 0xFF: {}", all_ff);
        for byte in 0..32 {
            print!("{:02X}", vram[tile1023_off + byte]);
        }
        println!();
    }

    // Dump first 32 BG palette entries
    println!("\n=== BG Palette 0-31 ===");
    for i in 0..32 {
        let off = i * 2;
        let c = u16::from_le_bytes([pal[off], pal[off + 1]]);
        if c != 0 {
            let r = (c & 0x1F) as u32 * 255 / 31;
            let g = ((c >> 5) & 0x1F) as u32 * 255 / 31;
            let b = ((c >> 10) as u32) * 255 / 31;
            println!("  BG Pal[{}]={:#06X} rgb=({},{},{})", i, c, r, g, b);
        }
    }

    // Check OBJ palette
    println!("\n=== OBJ Palette 0-15 ===");
    for i in 0..16 {
        let off = 0x200 + i * 2;
        let c = u16::from_le_bytes([pal[off], pal[off + 1]]);
        if c != 0 {
            let r = (c & 0x1F) as u32 * 255 / 31;
            let g = ((c >> 5) & 0x1F) as u32 * 255 / 31;
            let b = ((c >> 10) as u32) * 255 / 31;
            println!("  OBJ Pal[{}]={:#06X} rgb=({},{},{})", i, c, r, g, b);
        }
    }

    // VRAM stats
    let vram = gba.ppu().vram();
    let mut nz_bg = 0;
    let mut nz_obj = 0;
    for (i, &b) in vram.iter().enumerate() {
        if b != 0 {
            if i < 0x10000 {
                nz_bg += 1;
            } else {
                nz_obj += 1;
            }
        }
    }
    println!(
        "\nVRAM BG non-zero: {}/{} OBJ non-zero: {}/{}",
        nz_bg,
        0x10000,
        nz_obj,
        vram.len() - 0x10000
    );
}

use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }

    let ppu = gba.ppu();
    let dispcnt = ppu.get_dispcnt();
    let mode = dispcnt & 7;
    println!("DISPCNT: {:04X}  mode={}", dispcnt, mode);
    println!("Enabled layers:");
    for bg in 0..4 {
        if dispcnt & (1 << (8 + bg)) != 0 {
            let bgcnt = ppu.get_bgcnt(bg);
            let priority = bgcnt & 3;
            let char_base = ((bgcnt >> 2) & 0x3) * 0x4000;
            let screen_base = ((bgcnt >> 8) & 0x1F) * 0x800;
            let is_8bpp = (bgcnt & 0x80) != 0;
            let bg_size = (bgcnt >> 14) & 3;
            let hofs = ppu.get_bg_hofs(bg);
            let vofs = ppu.get_bg_vofs(bg);
            println!(
                "  BG{}: bgcnt={:04X} pri={} char_base={:05X} screen_base={:05X} {}bpp size={} hofs={} vofs={}",
                bg, bgcnt, priority, char_base, screen_base,
                if is_8bpp { "8" } else { "4" },
                bg_size, hofs, vofs
            );
        }
    }
    if dispcnt & (1 << 12) != 0 {
        println!("  OBJ: enabled");
    }
    println!();

    let vram = ppu.vram();

    let pal = gba.mem().palette();
    let backdrop = u16::from_le_bytes([pal[0], pal[1]]);
    println!(
        "Backdrop color: {:04X} (RGB5: R={} G={} B={})",
        backdrop,
        backdrop & 0x1F,
        (backdrop >> 5) & 0x1F,
        (backdrop >> 10) & 0x1F
    );

    println!("\nPalette first 16 colors:");
    for i in 0..16 {
        let c = u16::from_le_bytes([pal[i * 2], pal[i * 2 + 1]]);
        print!(
            "  [{:2}] {:04X}(R={} G={} B={})",
            i,
            c,
            c & 0x1F,
            (c >> 5) & 0x1F,
            (c >> 10) & 0x1F
        );
        if i % 4 == 3 {
            println!();
        }
    }

    for bg in 0..4 {
        if dispcnt & (1 << (8 + bg)) == 0 {
            continue;
        }
        let bgcnt = ppu.get_bgcnt(bg);
        let char_base = ((bgcnt >> 2) & 0x3) as usize * 0x4000;
        let screen_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;

        println!("\n=== BG{} screen entries (first 32x32) ===", bg);
        let mut nonzero_tiles = 0u32;
        let mut tile_hist: std::collections::HashMap<u16, u32> = std::collections::HashMap::new();
        for ty in 0..32 {
            for tx in 0..32 {
                let off = screen_base + (ty * 32 + tx) * 2;
                let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
                let tile_num = entry & 0x3FF;
                let palette = (entry >> 12) & 0xF;
                if entry != 0 {
                    nonzero_tiles += 1;
                }
                *tile_hist.entry(tile_num).or_insert(0) += 1;
            }
        }
        println!("  Non-zero entries: {}/1024", nonzero_tiles);
        let mut sorted: Vec<_> = tile_hist.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        print!("  Top 10 tile nums: ");
        for (tn, count) in sorted.iter().take(10) {
            print!("{}({}x) ", tn, count);
        }
        println!();

        let is_8bpp = (bgcnt & 0x80) != 0;
        let tile_size = if is_8bpp { 64 } else { 32 };

        let max_tile = sorted.iter().map(|(tn, _)| **tn).max().unwrap_or(0);
        let tile_data_end = char_base + (max_tile as usize + 1) * tile_size;
        let mut zero_tiles = 0u32;
        let mut nonzero_data = 0u32;
        for tn in 0..=max_tile {
            let base = char_base + tn as usize * tile_size;
            let mut all_zero = true;
            for i in 0..tile_size {
                if base + i < vram.len() && vram[base + i] != 0 {
                    all_zero = false;
                    nonzero_data += 1;
                }
            }
            if all_zero {
                zero_tiles += 1;
            }
        }
        println!("  Max tile num: {}", max_tile);
        println!(
            "  Tile data range: {:05X}-{:05X}",
            char_base,
            tile_data_end.min(vram.len())
        );
        println!("  Zero tiles (all data=0): {}/{}", zero_tiles, max_tile + 1);
        println!("  Non-zero bytes in tile data: {}", nonzero_data);

        println!("\n  Sample tile data (tile 0, 1, 2):");
        for tn in 0..3 {
            let base = char_base + tn as usize * tile_size;
            print!("    Tile {}: ", tn);
            for i in 0..tile_size.min(16) {
                if base + i < vram.len() {
                    print!("{:02X} ", vram[base + i]);
                }
            }
            println!();
        }
    }

    println!("\n=== VRAM usage summary ===");
    let mut nonzero_regions = 0;
    for region in 0..6 {
        let start = region * 0x4000;
        let end = (region + 1) * 0x4000;
        let count: usize = vram[start..end].iter().filter(|&&b| b != 0).count();
        if count > 0 {
            nonzero_regions += 1;
            println!(
                "  {:05X}-{:05X}: {} nonzero bytes ({:.1}%)",
                start,
                end,
                count,
                count as f64 / 0x4000 as f64 * 100.0
            );
        }
    }
    if nonzero_regions == 0 {
        println!("  VRAM is all zeros!");
    }
}

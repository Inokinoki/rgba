use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    // Intercept DMA and log VRAM transfers
    // Instead, let me check: after 1000 frames, where does tile data exist?
    // We know tiles 0-115 have data. Let me check if tiles 187-255 have data.

    for _ in 0..1000 {
        gba.run_frame();
    }

    let vram = gba.mem().vram();

    // Check tiles 100-255
    println!("=== Tiles 100-255 data check ===");
    for tile in (100..256).step_by(10) {
        let off = tile * 32;
        let mut has_data = false;
        let mut sum = 0u32;
        for byte in 0..32 {
            sum += vram[off + byte] as u32;
            if vram[off + byte] != 0 {
                has_data = true;
            }
        }
        print!(
            "Tile {}: {} sum={}",
            tile,
            if has_data { "DATA" } else { "empty" },
            sum
        );
        if has_data {
            print!(" bytes=");
            for byte in 0..8 {
                print!("{:02X}", vram[off + byte]);
            }
        }
        println!();
    }

    // Key insight: let me check if BG0 references use tile numbers that
    // should wrap within a 256-tile space
    // In 4bpp mode, with char_base=0, tile N is at offset N*32
    // The screen entry has 10 bits for tile number (0-1023)
    // But maybe the game expects only 256 tiles per char block?

    // Let me check what tile numbers are actually in BG0's screen entries
    let io = gba.mem().io();
    let bg0cnt = u16::from_le_bytes([io[0x08], io[0x09]]);
    let screen_base = ((bg0cnt >> 8) & 0x1F) as usize * 0x800;

    let mut tile_hist: std::collections::HashMap<u16, usize> = std::collections::HashMap::new();
    for i in 0..2048 {
        let off = screen_base + i * 2;
        if off + 1 < vram.len() {
            let e = u16::from_le_bytes([vram[off], vram[off + 1]]);
            let t = e & 0x3FF;
            *tile_hist.entry(t).or_insert(0) += 1;
        }
    }

    let mut tiles: Vec<(u16, usize)> = tile_hist.into_iter().collect();
    tiles.sort_by(|a, b| b.1.cmp(&a.1));
    println!("\n=== BG0 tile frequency ===");
    for (tile, count) in tiles.iter().take(15) {
        let data_off = *tile as usize * 32;
        let has_data =
            data_off + 32 <= vram.len() && vram[data_off..data_off + 32].iter().any(|&b| b != 0);
        println!("  Tile {}: {} entries, has_data={}", tile, count, has_data);
    }

    // Check: do ANY BG0 tiles have data?
    let tiles_with_data: Vec<u16> = tiles
        .iter()
        .filter(|(t, _)| {
            let off = *t as usize * 32;
            off + 32 <= vram.len() && vram[off..off + 32].iter().any(|&b| b != 0)
        })
        .map(|(t, _)| *t)
        .collect();
    println!("\nBG0 tiles with data: {:?}", tiles_with_data);

    // The real question: what tiles have data in VRAM AND are used by ANY BG layer?
    println!("\n=== All tiles with data, used by any BG ===");
    for tile in 0..1024u16 {
        let off = tile as usize * 32;
        if off + 32 > vram.len() {
            break;
        }
        let has_data = vram[off..off + 32].iter().any(|&b| b != 0);
        if has_data {
            // Check if referenced by any BG
            for bg in 0..4 {
                let bgcnt_off = 0x08 + bg * 2;
                let bgcnt = u16::from_le_bytes([io[bgcnt_off], io[bgcnt_off + 1]]);
                let sb = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
                for i in 0..2048 {
                    let off2 = sb + i * 2;
                    if off2 + 1 < vram.len() {
                        let e = u16::from_le_bytes([vram[off2], vram[off2 + 1]]);
                        if (e & 0x3FF) == tile {
                            println!(
                                "  Tile {} has data, used by BG{} (entry={:#06X})",
                                tile, bg, e
                            );
                            break;
                        }
                    }
                }
            }
        }
    }
}

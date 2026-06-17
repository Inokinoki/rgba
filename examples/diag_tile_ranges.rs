use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.sync_ppu_full();

    let vram = gba.mem.vram();

    // Show which tiles have data, grouped by ranges
    let mut ranges: Vec<(u16, u16)> = Vec::new(); // (start, end) of contiguous non-zero ranges
    let mut in_range = false;
    let mut range_start = 0u16;
    for tile_id in 0..1024u16 {
        let offset = tile_id as usize * 32;
        let all_zero = vram[offset..offset + 32].iter().all(|&b| b == 0);
        if !all_zero && !in_range {
            range_start = tile_id;
            in_range = true;
        } else if all_zero && in_range {
            ranges.push((range_start, tile_id - 1));
            in_range = false;
        }
    }
    if in_range {
        ranges.push((range_start, 1023));
    }

    eprintln!("Non-zero tile ranges:");
    for (start, end) in &ranges {
        eprintln!("  tiles {}-{} ({} tiles)", start, end, end - start + 1);
    }

    // Show the first screen entry for each row of BG0
    let bg0_screen = 0x0C000usize;
    let hofs = gba.ppu.get_bg_hofs(0);
    let vofs = gba.ppu.get_bg_vofs(0);
    let first_col = (hofs / 8) as usize;

    eprintln!("\nBG0 first visible tile per row (col={}):", first_col);
    for row in 0..20 {
        let tile_row = row + (vofs as usize / 8);
        let entry_idx = tile_row * 32 + first_col;
        let off = bg0_screen + entry_idx * 2;
        let se = vram[off] as u16 | ((vram[off + 1] as u16) << 8);
        let tile = se & 0x3FF;
        let pal = (se >> 12) & 0xF;
        eprintln!("  row {:2}: tile={} pal={}", row, tile, pal);
    }

    // Count: how many screen entries in visible area reference zero tiles?
    let mut zero_count = 0;
    let mut nonzero_count = 0;
    for row in 0..20 {
        for col in 0..30 {
            let tile_col = col + first_col;
            let tile_row = row + (vofs as usize / 8);
            let block = if tile_col < 32 { 0 } else { 1 };
            let local_col = tile_col % 32;
            let entry_idx = tile_row * 32 + local_col;
            let off = bg0_screen + block * 0x800 + entry_idx * 2;
            let se = vram[off] as u16 | ((vram[off + 1] as u16) << 8);
            let tile = se & 0x3FF;
            let tile_offset = tile as usize * 32;
            let all_zero = vram[tile_offset..tile_offset + 32].iter().all(|&b| b == 0);
            if all_zero {
                zero_count += 1;
            } else {
                nonzero_count += 1;
            }
        }
    }
    eprintln!(
        "\nVisible area: {} zero tiles, {} non-zero tiles (out of 600)",
        zero_count, nonzero_count
    );

    // Check tiles 512-1023 which is the second half of char_base
    let mut upper_nonzero = 0;
    for tile_id in 512..1024u16 {
        let offset = tile_id as usize * 32;
        let all_zero = vram[offset..offset + 32].iter().all(|&b| b == 0);
        if !all_zero {
            upper_nonzero += 1;
        }
    }
    eprintln!("Non-zero tiles in 512-1023 range: {}", upper_nonzero);
}

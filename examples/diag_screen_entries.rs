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

    // Dump ALL screen entries for all 4 BGs
    let bgs = [
        (0, 0xC000, "BG0"),
        (1, 0xD000, "BG1"),
        (2, 0xE000, "BG2"),
        (3, 0xF000, "BG3"),
    ];

    for (bg_idx, base, name) in &bgs {
        let hofs = gba.ppu.get_bg_hofs(*bg_idx);
        let vofs = gba.ppu.get_bg_vofs(*bg_idx);
        let bgcnt = gba.ppu.get_bgcnt(*bg_idx);
        let priority = bgcnt & 3;
        let size = (bgcnt >> 14) & 3;
        eprintln!(
            "\n{}: priority={}, hofs={}, vofs={}, size={}, screen_base=0x{:04X}",
            name, priority, hofs, vofs, size, base
        );

        let (cols, rows) = match size {
            0 => (32, 32),
            1 => (64, 32),
            2 => (32, 64),
            3 => (64, 64),
            _ => (32, 32),
        };

        // Calculate visible range
        let vis_col_start = (hofs / 8) as usize;
        let vis_col_end = vis_col_start + 30;
        let vis_row_start = (vofs / 8) as usize;
        let vis_row_end = vis_row_start + 20;

        // Count tile IDs referenced in visible area
        let mut tile_id_counts: std::collections::BTreeMap<u16, usize> =
            std::collections::BTreeMap::new();

        for row in vis_row_start..vis_row_end.min(rows) {
            for col in vis_col_start..vis_col_end.min(cols) {
                // Handle screen entry addressing for 64-wide maps
                let (map_row, map_col, block_offset) = if cols == 64 {
                    let r = row % 32;
                    let c = col % 32;
                    let block = if row >= 32 { 2 } else { 0 } + if col >= 32 { 1 } else { 0 };
                    (r, c, block * 0x800)
                } else if rows == 64 {
                    let r = row % 32;
                    let block = if row >= 32 { 2 } else { 0 };
                    (r, col, block * 0x800)
                } else {
                    (row, col, 0)
                };

                let entry_addr = base + block_offset + (map_row * 32 + map_col) * 2;
                if entry_addr + 1 < vram.len() {
                    let entry = u16::from_le_bytes([vram[entry_addr], vram[entry_addr + 1]]);
                    let tile = entry & 0x3FF;
                    *tile_id_counts.entry(tile).or_insert(0) += 1;
                }
            }
        }

        eprintln!(
            "  Visible area: cols {}-{}, rows {}-{}",
            vis_col_start,
            vis_col_end - 1,
            vis_row_start,
            vis_row_end - 1
        );
        eprintln!(
            "  Unique tile IDs in visible area: {}",
            tile_id_counts.len()
        );

        // Show tile IDs sorted by count
        let mut sorted: Vec<_> = tile_id_counts.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        eprintln!("  Top 20 tile IDs:");
        for (tid, count) in sorted.iter().take(20) {
            let off = **tid as usize * 32;
            let has_data = off + 32 <= vram.len() && vram[off..off + 32].iter().any(|&b| b != 0);
            eprintln!(
                "    tile {}: {} refs, {}",
                tid,
                count,
                if has_data { "HAS DATA" } else { "EMPTY" }
            );
        }
    }

    // Also dump screen entries for BG0 around the visible area
    eprintln!("\nBG0 screen entries (first 8 rows, visible cols):");
    let hofs = gba.ppu.get_bg_hofs(0);
    let base = 0xC000;
    let vis_col = (hofs / 8) as usize;
    for row in 0..8 {
        for col in vis_col..vis_col + 32 {
            let (map_row, map_col, block_offset) = if col >= 32 {
                (row, col - 32, 0x800)
            } else {
                (row, col, 0)
            };
            let entry_addr = base + block_offset + (map_row * 32 + map_col) * 2;
            if entry_addr + 1 < vram.len() {
                let entry = u16::from_le_bytes([vram[entry_addr], vram[entry_addr + 1]]);
                let tile = entry & 0x3FF;
                let pal = (entry >> 12) & 0xF;
                if col % 8 == 0 {
                    eprint!("\n  row={}: ", row);
                }
                eprint!("{:4}(p{}) ", tile, pal);
            }
        }
        eprintln!();
    }
}

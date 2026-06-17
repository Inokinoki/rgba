use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Enable VRAM write logging
    gba.mem.vram_log_enabled = true;

    for i in 0..192 {
        gba.run_frame_parallel(&mut fb);
    }

    let writes = &gba.mem.vram_write_log;
    eprintln!("Total VRAM writes in first 192 frames: {}", writes.len());

    // Find the frame with the most writes
    let mut frame_writes: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for &(_, pc, _) in writes {
        let frame = pc; // approximate frame from order
        *frame_writes.entry(frame).or_insert(0) += 1;
    }

    // Group by PC ranges to identify decompression functions
    let mut pc_ranges: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for &(_, pc, _) in writes {
        let page = pc & 0xFFFFF000;
        *pc_ranges.entry(page).or_insert(0) += 1;
    }
    let mut sorted: Vec<_> = pc_ranges.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    eprintln!("VRAM writes by PC page:");
    for (page, count) in sorted.iter().take(5) {
        eprintln!("  0x{:08X}: {} writes", page, count);
    }

    // Check tile 394 specifically - it's used 177 times but all zeros
    let vram = gba.mem.vram();
    let char_base = 0u32;
    let tile_394_offset = char_base + 394 * 32;
    eprintln!("\nTile 394 @0x{:05X}:", tile_394_offset);
    let mut all_zero = true;
    for b in 0..32 {
        if vram[(tile_394_offset + b) as usize] != 0 {
            all_zero = false;
            break;
        }
    }
    eprintln!("  All zeros: {}", all_zero);

    // Find VRAM writes to tile 394 area
    let tile_start = tile_394_offset as u32;
    let tile_end = tile_start + 32;
    let writes_to_tile: Vec<_> = writes
        .iter()
        .filter(|&&(addr, _, _)| addr >= tile_start && addr < tile_end)
        .collect();
    eprintln!("  Writes to tile 394 area: {}", writes_to_tile.len());
    for &(addr, pc, val) in writes_to_tile.iter().take(5) {
        eprintln!("    addr=0x{:05X} pc=0x{:08X} val=0x{:02X}", addr, pc, val);
    }

    // Check what tile data SHOULD be by reading from ROM
    // The game ROM contains compressed tile data. Let me check if there's
    // a reference tile at a known location.
    // For now, let me check a tile that HAS data (tile 473 was in the first screen entry)
    let tile_473_offset = char_base + 473 * 32;
    eprintln!("\nTile 473 @0x{:05X}:", tile_473_offset);
    let hex: String = (0..32)
        .map(|b| format!("{:02X}", vram[(tile_473_offset + b) as usize]))
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!("  {}", hex);

    // Check if tile 0 has been overwritten (it was non-zero at frame 121)
    let tile_0_offset = char_base + 0 * 32;
    eprintln!("\nTile 0 @0x{:05X}:", tile_0_offset);
    let hex: String = (0..32)
        .map(|b| format!("{:02X}", vram[(tile_0_offset + b) as usize]))
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!("  {}", hex);

    // Check first few screen entries for BG0 at the visible area
    let bg0_screen = 0x0C000usize;
    let hofs = 208u16;
    let first_tile_col = (hofs / 8) as usize; // = 26
    eprintln!(
        "\nBG0 visible screen entries (row 0, cols {}-{}):",
        first_tile_col,
        first_tile_col + 29
    );
    for col in first_tile_col..first_tile_col + 30 {
        let off = bg0_screen + col * 2;
        let se = vram[off] as u16 | ((vram[off + 1] as u16) << 8);
        let tile = se & 0x3FF;
        let pal = (se >> 12) & 0xF;
        eprintln!("  col={:2}: tile={} pal={}", col, tile, pal);
    }
}

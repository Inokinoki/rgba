use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    // Trace VRAM writes during the critical frames (3-10)
    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();
    gba.mem_mut().vram_write_log.reserve(500_000);

    // Increase log limit
    // The limit is in mem.rs line 799: len() < 100_000

    // Run frame 3 (where VRAM writes start)
    for frame in 0..15 {
        for _ in 0..228 {
            gba.run_scanline();
        }

        let log = &gba.mem().vram_write_log;
        if !log.is_empty() {
            let mut tile_writes: Vec<(u32, u32, u8)> = log
                .iter()
                .filter(|&&(addr, _, _)| {
                    (addr & 0x0FFFFFFF) >= 0x06000000 && (addr & 0x0FFFFFFF) < 0x06008000
                })
                .cloned()
                .collect();

            if !tile_writes.is_empty() {
                tile_writes.sort_by_key(|&(addr, _, _)| addr);
                let first = tile_writes.first().unwrap();
                let last = tile_writes.last().unwrap();
                let first_off = (first.0 & 0x0FFFFFFF) - 0x06000000;
                let last_off = (last.0 & 0x0FFFFFFF) - 0x06000000;

                println!(
                    "Frame {}: {} tile writes, range {:05X}-{:05X}",
                    frame,
                    tile_writes.len(),
                    first_off,
                    last_off
                );

                // Check for gaps in tile data
                let mut prev_off = 0xFFFFFFFFu32;
                let mut gaps = 0;
                for &(addr, _, _) in &tile_writes {
                    let off = (addr & 0x0FFFFFFF) - 0x06000000;
                    if prev_off != 0xFFFFFFFF && off > prev_off + 2 && off - prev_off > 0x100 {
                        println!(
                            "  GAP: {:05X} -> {:05X} ({} bytes)",
                            prev_off,
                            off,
                            off - prev_off
                        );
                        gaps += 1;
                    }
                    prev_off = off;
                }
                if gaps == 0 {
                    println!("  No large gaps in tile writes");
                }
            }
        }
        gba.mem_mut().vram_write_log.clear();
    }
}

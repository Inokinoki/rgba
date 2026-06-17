use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.mem_mut().vram_log_enabled = true;

    let mut fb = vec![0u32; 240 * 160];
    for frame in 0..195 {
        gba.mem_mut().vram_write_log.clear();
        gba.run_frame_parallel(&mut fb);

        let log = &gba.mem().vram_write_log;
        if !log.is_empty() && frame >= 185 {
            let mut tile_writes: Vec<(u32, u32)> = log
                .iter()
                .filter_map(|&(addr, pc, _val)| {
                    let off = (addr & 0x0FFFFFFF);
                    if off >= 0x06000000 && off < 0x06008000 {
                        Some((off - 0x06000000, pc))
                    } else {
                        None
                    }
                })
                .collect();

            let screen_writes: Vec<(u32, u32)> = log
                .iter()
                .filter_map(|&(addr, pc, _val)| {
                    let off = (addr & 0x0FFFFFFF);
                    if off >= 0x0600C000 && off < 0x06010000 {
                        Some((off - 0x06000000, pc))
                    } else {
                        None
                    }
                })
                .collect();

            if !tile_writes.is_empty() {
                tile_writes.sort_by_key(|&(a, _)| a);
                let first = tile_writes.first().unwrap().0;
                let last = tile_writes.last().unwrap().0;

                // Check if tile 1023 area (0x7FE0-0x7FFF) gets written
                let tile_1023_writes: Vec<_> = tile_writes
                    .iter()
                    .filter(|&&(a, _)| a >= 0x7FE0 && a <= 0x7FFF)
                    .collect();

                println!(
                    "Frame {}: {} tile writes [{:05X}-{:05X}], tile 1023 area writes: {}",
                    frame,
                    tile_writes.len(),
                    first,
                    last,
                    tile_1023_writes.len()
                );

                // Show last 5 tile writes
                for &(a, pc) in tile_writes.iter().rev().take(5) {
                    println!("  Tile write: {:05X} from PC={:08X}", a, pc);
                }
            }

            if !screen_writes.is_empty() {
                println!("Frame {}: {} screen writes", frame, screen_writes.len());
            }
        }
    }
}

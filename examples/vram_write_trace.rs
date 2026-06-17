use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.reserve(1_000_000);

    for frame in 0..10 {
        for _ in 0..228 {
            gba.run_scanline();
        }
        let log = &gba.mem().vram_write_log;
        let count = log.len();
        if count > 0 {
            let mut vram_writes: Vec<(u32, u32, u8)> = log.clone();
            vram_writes.sort_by_key(|&(addr, _, _)| addr);

            let mut range_start = vram_writes[0].0;
            let mut range_end = range_start;
            let mut range_count = 0;

            println!("=== Frame {} VRAM writes: {} total ===", frame, count);
            for &(addr, pc, val) in &vram_writes {
                if addr > range_end + 2 {
                    if range_count > 0 {
                        println!(
                            "  {:05X}-{:05X}: {} writes",
                            range_start & 0x1FFFF,
                            range_end & 0x1FFFF,
                            range_count
                        );
                    }
                    range_start = addr;
                    range_count = 0;
                }
                range_end = addr;
                range_count += 1;
            }
            if range_count > 0 {
                println!(
                    "  {:05X}-{:05X}: {} writes",
                    range_start & 0x1FFFF,
                    range_end & 0x1FFFF,
                    range_count
                );
            }
        }
        gba.mem_mut().vram_write_log.clear();
    }
}

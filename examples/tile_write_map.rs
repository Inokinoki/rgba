use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();

    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let log = &gba.mem().vram_write_log;

    let target_tiles = [394, 403, 412, 420, 473, 482, 491, 499];
    for &t in &target_tiles {
        let start_off = t * 32;
        let end_off = start_off + 32;
        let mut writes = 0;
        for (addr, _pc, _val) in log {
            let offset = (addr - 0x0600_0000) as usize;
            if offset >= start_off && offset < end_off {
                writes += 1;
            }
        }
        if writes > 0 {
            println!(
                "Tile {}: {} writes (offset {:#X}-{:#X})",
                t,
                writes,
                start_off,
                end_off - 1
            );
        }
    }

    let mut cb0_writes_by_tile = vec![0u32; 512];
    for (addr, _pc, _val) in log {
        let offset = (addr - 0x0600_0000) as usize;
        if offset < 0x4000 {
            let tile = offset / 32;
            cb0_writes_by_tile[tile] += 1;
        }
    }

    println!("\nChar block 0 tile write distribution:");
    let mut written = 0;
    let mut not_written = 0;
    for (tile, &count) in cb0_writes_by_tile.iter().enumerate() {
        if count > 0 {
            written += 1;
        } else {
            not_written += 1;
        }
    }
    println!("Tiles with writes: {}", written);
    println!("Tiles without writes: {}", not_written);

    println!("\nWritten tile ranges:");
    let mut in_range = false;
    let mut range_start = 0;
    for (tile, &count) in cb0_writes_by_tile.iter().enumerate() {
        if count > 0 && !in_range {
            range_start = tile;
            in_range = true;
        } else if count == 0 && in_range {
            println!("  Tiles {}-{}", range_start, tile - 1);
            in_range = false;
        }
    }
    if in_range {
        println!("  Tiles {}-511", range_start);
    }
}

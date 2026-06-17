use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.mem_mut().vram_log_enabled = true;

    for frame in 0..200 {
        for _sl in 0..228 {
            gba.run_scanline();
        }
        if frame % 50 == 0 {
            eprintln!("Frame {}", frame);
        }
    }

    let log = &gba.mem().vram_write_log;
    eprintln!("Total VRAM writes: {}", log.len());

    let mut first_tile_addr = 0xFFFFFFFFu32;
    let mut last_tile_addr = 0u32;
    let mut tile_write_count = 0u32;

    for &(addr, pc, _val) in log {
        if addr >= 0x06000000 && addr < 0x0600C000 {
            if addr < first_tile_addr {
                first_tile_addr = addr;
            }
            if addr > last_tile_addr {
                last_tile_addr = addr;
            }
            tile_write_count += 1;
        }
    }

    println!(
        "Tile area (0x06000000-0x0600BFFF): {} writes, range {:08X}-{:08X}",
        tile_write_count, first_tile_addr, last_tile_addr
    );

    let first_tile = (first_tile_addr - 0x06000000) / 32;
    let last_tile = (last_tile_addr - 0x06000000) / 32;
    println!("Tile range: {}-{}", first_tile, last_tile);

    let mut screen_writes = 0u32;
    let mut screen_first = 0xFFFFFFFFu32;
    let mut screen_last = 0u32;
    for &(addr, _pc, _val) in log {
        if addr >= 0x0600C000 && addr < 0x06010000 {
            if addr < screen_first {
                screen_first = addr;
            }
            if addr > screen_last {
                screen_last = addr;
            }
            screen_writes += 1;
        }
    }
    println!(
        "Screen block area (0x0600C000-0x0600FFFF): {} writes, range {:08X}-{:08X}",
        screen_writes, screen_first, screen_last
    );

    let mut pc_hist: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    for &(addr, pc, _val) in log {
        if addr >= 0x06000000 && addr < 0x0600C000 {
            *pc_hist.entry(pc).or_insert(0) += 1;
        }
    }

    println!("\nPC histogram for tile area writes:");
    let mut pcs: Vec<_> = pc_hist.iter().collect();
    pcs.sort_by_key(|(_, &cnt)| std::cmp::Reverse(cnt));
    for (pc, cnt) in pcs.iter().take(20) {
        println!("  PC {:08X}: {} writes", pc, cnt);
    }

    println!("\nPer-tile write count (first 120 tiles):");
    let mut tile_counts: Vec<u32> = vec![0; 120];
    for &(addr, _pc, _val) in log {
        if addr >= 0x06000000 && addr < 0x06000F00 {
            let tile = (addr - 0x06000000) / 32;
            if (tile as usize) < tile_counts.len() {
                tile_counts[tile as usize] += 1;
            }
        }
    }
    for t in (0..120).step_by(10) {
        let end = (t + 10).min(120);
        let line: Vec<String> = (t..end)
            .map(|i| format!("{}:{}", i, tile_counts[i]))
            .collect();
        println!("  {}", line.join(" "));
    }
}

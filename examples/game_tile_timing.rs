use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.mem.vram_log_enabled = true;
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    // Check VRAM at each frame to see when tile data appears
    for frame in 0..20 {
        gba.run_frame();

        let vram = gba.mem().vram();

        // Check tile 394
        let tile_off = 394 * 32;
        let mut sum = 0u32;
        for i in 0..32 {
            sum += vram[tile_off + i] as u32;
        }

        // Check tile 0 (we know this has data later)
        let tile0_sum: u32 = vram[0..32].iter().map(|&b| b as u32).sum();

        // Check total BG tile data
        let mut total = 0u32;
        for i in 0..0x4000 {
            total += vram[i] as u32;
        }

        if frame < 10 || sum > 0 || tile0_sum > 0 {
            println!(
                "Frame {}: tile394_sum={} tile0_sum={} bg_total={}",
                frame, sum, tile0_sum, total
            );
        }
    }

    // After 20 frames, check VRAM write log for tile area
    let log = &gba.mem.vram_write_log;
    let mut tile_writes = 0;
    let mut max_tile_addr = 0usize;
    for &(addr, pc, val) in log {
        let offset = (addr & 0x1FFFF) as usize;
        if offset < 0x4000 {
            tile_writes += 1;
            max_tile_addr = max_tile_addr.max(offset);
        }
    }
    println!("\nAfter 20 frames:");
    println!("  VRAM writes to tile area: {}", tile_writes);
    println!("  Max tile write address: {:#06X}", max_tile_addr);
    println!("  VRAM log total: {}", log.len());
}

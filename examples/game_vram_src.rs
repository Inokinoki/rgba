use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.mem.vram_log_enabled = true;
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    // Run only a few frames and check DMA
    for frame in 0..10 {
        gba.run_frame();
    }

    let log = &gba.mem.vram_write_log;
    println!("After 10 frames: {} VRAM writes", log.len());

    // Check unique PCs that write to VRAM
    let mut pc_counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for &(_addr, pc, _val) in log {
        *pc_counts.entry(pc).or_insert(0) += 1;
    }
    let mut sorted_pcs: Vec<_> = pc_counts.iter().collect();
    sorted_pcs.sort_by(|a, b| b.1.cmp(a.1));
    println!("\nTop 10 PCs writing to VRAM:");
    for (pc, count) in sorted_pcs.iter().take(10) {
        let real_pc = **pc << 1;
        println!(
            "  PC={:#010X} (real={:#010X}): {} writes",
            pc, real_pc, count
        );
    }

    // Check where tile data (0x0000-0x1FFF) comes from
    let mut tile_pc_counts: std::collections::HashMap<u32, usize> =
        std::collections::HashMap::new();
    for &(addr, pc, _val) in log {
        let offset = addr & 0x1FFFF;
        if offset < 0x2000 {
            *tile_pc_counts.entry(pc).or_insert(0) += 1;
        }
    }
    println!("\nPCs writing to tile area (0x0000-0x1FFF):");
    let mut sorted: Vec<_> = tile_pc_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    for (pc, count) in sorted.iter().take(5) {
        let real_pc = **pc << 1;
        println!(
            "  PC={:#010X} (real={:#010X}): {} writes",
            pc, real_pc, count
        );
    }

    // Now check: are there ANY VRAM writes between frames?
    // Maybe tile data is loaded during DMA
    // Let me check if SWI CpuSet writes to VRAM
    // CpuSet is SWI 0x0B

    // Actually, let me check a completely different approach:
    // What if the game writes tile data via 16-bit or 32-bit writes,
    // but VRAM byte expansion messes it up?

    // Let me trace what happens with write_half to VRAM
    println!("\n=== Checking write_half path for VRAM ===");

    // VRAM write_half should call write_byte_internal twice
    // Let's see if write_byte works correctly

    // Direct test: write a value and read it back
    drop(gba); // borrow issue
    let mut gba2 = Gba::new();
    gba2.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    // Write via halfword
    gba2.write_half(0x06003140, 0x1234);
    let vram = gba2.mem().vram();
    println!(
        "write_half(0x06003140, 0x1234): vram[0x3140]={:#04X} vram[0x3141]={:#04X}",
        vram[0x3140], vram[0x3141]
    );

    // Write via word
    gba2.mem.write_word(0x06003144, 0xDEADBEEF);
    let vram = gba2.mem().vram();
    println!("write_word(0x06003144, 0xDEADBEEF): vram[0x3144]={:#04X} vram[0x3145]={:#04X} vram[0x3146]={:#04X} vram[0x3147]={:#04X}",
        vram[0x3144], vram[0x3145], vram[0x3146], vram[0x3147]);
}

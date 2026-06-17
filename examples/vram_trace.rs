use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    gba.mem_mut().vram_log_enabled = true;

    let mut framebuffer = vec![0u32; 240 * 160];
    for _ in 0..240 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    let log = &gba.mem().vram_write_log;
    println!("Total VRAM writes in 240 frames: {}", log.len());

    let vram = gba.mem().vram();

    for region in 0..6 {
        let base = region * 0x4000;
        let end = base + 0x4000;
        if end > vram.len() {
            break;
        }
        let nonzero = vram[base..end].iter().filter(|&&b| b != 0).count();
        let writes_in_region = log
            .iter()
            .filter(|(a, _, _)| {
                let off = (*a as usize) & 0x1FFFF;
                off >= base && off < end
            })
            .count();
        println!(
            "Region {:#06X}-{:#06X}: {} nonzero bytes, {} writes",
            base, end, nonzero, writes_in_region
        );
    }

    for tile_base in [0x0000usize, 0x4000, 0x8000, 0xC000] {
        let mut count = 0u32;
        let max_tiles = if tile_base == 0xC000 { 512 } else { 1024 };
        for tile in 0..max_tiles {
            let off = tile_base + tile * 32;
            if off + 32 > vram.len() {
                break;
            }
            if (0..32).any(|i| vram[off + i] != 0) {
                count += 1;
            }
        }
        if count > 0 {
            println!("Tile data at {:#06X}: {} tiles have data", tile_base, count);
        }
    }

    let mut pc_counts = std::collections::HashMap::new();
    for &(_, pc, _) in log {
        *pc_counts.entry(pc).or_insert(0u32) += 1;
    }
    let mut pcs: Vec<_> = pc_counts.iter().collect();
    pcs.sort_by(|a, b| b.1.cmp(a.1));
    println!("\nTop 10 PCs writing to VRAM:");
    for (&pc, &count) in pcs.iter().take(10) {
        println!("  PC={:#010X}: {} writes", pc * 2, count);
    }

    println!("\nLast 30 VRAM writes:");
    for (addr, pc, val) in log.iter().rev().take(30) {
        let offset = (*addr as usize) & 0x1FFFF;
        let region = if offset < 0x10000 { "tile" } else { "OBJ" };
        println!(
            "  {:#010X} [{}+{:#X}] pc={:#010X} val={:#04X}",
            addr,
            region,
            offset,
            pc * 2,
            val
        );
    }
}

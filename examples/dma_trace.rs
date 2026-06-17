use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    gba.mem_mut().dma_log_enabled = true;

    let mut framebuffer = vec![0u32; 240 * 160];

    // Run 300 frames
    for _ in 0..300 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    let log = &gba.mem().dma_log;
    println!("DMA transfers: {}", log.len());
    
    // Filter for VRAM writes
    let vram_transfers: Vec<_> = log.iter().filter(|(_, _, dst, _, _)| 
        *dst >= 0x06000000 && *dst < 0x06018000
    ).collect();
    println!("DMA transfers to VRAM: {}", vram_transfers.len());
    
    for (i, &(ch, src, dst, count, size)) in log.iter().enumerate() {
        if i >= 50 { break; }
        let dst_region = if dst >= 0x06000000 && dst < 0x06018000 {
            format!("VRAM+{:#06X}", dst - 0x06000000)
        } else if dst >= 0x02000000 && dst < 0x02040000 {
            format!("EWRAM+{:#06X}", dst - 0x02000000)
        } else if dst >= 0x03000000 && dst < 0x03008000 {
            format!("IWRAM+{:#06X}", dst - 0x03000000)
        } else if dst >= 0x05000000 && dst < 0x05000400 {
            "PALETTE".to_string()
        } else if dst >= 0x07000000 && dst < 0x07000400 {
            "OAM".to_string()
        } else {
            format!("{:#010X}", dst)
        };
        println!("  DMA{}: src={:#010X} -> {} count={} size={}", 
                 ch, src, dst_region, count, size);
    }
    
    if log.len() > 50 {
        // Also show last 20
        println!("  ... ({} more) ...", log.len() - 50);
        for &(ch, src, dst, count, size) in log.iter().rev().take(20).rev() {
            let dst_region = if dst >= 0x06000000 && dst < 0x06018000 {
                format!("VRAM+{:#06X}", dst - 0x06000000)
            } else {
                format!("{:#010X}", dst)
            };
            println!("  DMA{}: src={:#010X} -> {} count={} size={}", 
                     ch, src, dst_region, count, size);
        }
    }
}

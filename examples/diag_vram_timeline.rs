use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Only log writes to tiles 344-472 area (VRAM offsets 0x2C00-0x3B20)
    // That's addresses 0x06002C00-0x06003B1F
    gba.mem.vram_log_enabled = true;
    // Increase log size temporarily by modifying log
    // We'll just check what the writes contain

    // Run only 5 frames to see early VRAM writes
    for i in 0..5 {
        gba.run_frame_parallel(&mut fb);
        gba.sync_ppu_full();
        let vram = gba.mem.vram();
        let mut nonzero = 0;
        for tid in 344..=472u32 {
            let off = tid as usize * 32;
            if vram[off..off + 32].iter().any(|&b| b != 0) {
                nonzero += 1;
            }
        }
        eprintln!("After frame {}: tiles 344-472 with data: {}", i, nonzero);
    }

    gba.mem.vram_write_log.clear();

    // Run frames 5-10
    for i in 5..10 {
        gba.run_frame_parallel(&mut fb);
        gba.sync_ppu_full();
        let vram = gba.mem.vram();
        let mut nonzero = 0;
        for tid in 344..=472u32 {
            let off = tid as usize * 32;
            if vram[off..off + 32].iter().any(|&b| b != 0) {
                nonzero += 1;
            }
        }
        eprintln!("After frame {}: tiles 344-472 with data: {}", i, nonzero);
    }

    // Check the values written to tiles 344-472 area
    eprintln!("\nWrites to tile 394 area (0x06003140-0x0600315F):");
    let mut writes_394: Vec<(u32, u8)> = Vec::new();
    for &(addr, pc, val) in &gba.mem.vram_write_log {
        let raw_offset = (addr - 0x0600_0000) % 0x2_0000;
        let offset = if raw_offset >= 0x1_8000 {
            raw_offset - 0x8000
        } else {
            raw_offset
        };
        if offset >= 394 * 32 && offset < 395 * 32 {
            writes_394.push((pc, val));
        }
    }
    eprintln!("  {} writes to tile 394", writes_394.len());
    for (pc, val) in writes_394.iter().take(40) {
        eprintln!("    PC=0x{:08X} val=0x{:02X}", pc, val);
    }

    // Now run many frames and check periodically
    gba.mem.vram_write_log.clear();
    gba.mem.vram_log_enabled = false;

    let mut frame_data_changes: Vec<(u32, u32, usize)> = Vec::new();
    for i in 10..240 {
        gba.run_frame_parallel(&mut fb);
        if i % 30 == 0 {
            gba.sync_ppu_full();
            let vram = gba.mem.vram();
            let mut nonzero = 0;
            for tid in 344..=472u32 {
                let off = tid as usize * 32;
                if vram[off..off + 32].iter().any(|&b| b != 0) {
                    nonzero += 1;
                }
            }
            frame_data_changes.push((i, nonzero, gba.mem.vram_write_log.len()));
            eprintln!("Frame {}: tiles 344-472 with data: {}", i, nonzero);
        }
    }

    // Final state
    gba.sync_ppu_full();
    let vram = gba.mem.vram();
    eprintln!("\n=== Final state (frame 240) ===");
    let mut total_nonzero = 0;
    for tid in 344..=472u32 {
        let off = tid as usize * 32;
        if vram[off..off + 32].iter().any(|&b| b != 0) {
            total_nonzero += 1;
        }
    }
    eprintln!("Tiles 344-472 with data: {}/129", total_nonzero);

    // Check what data is at tile 344 specifically
    let off = 344 * 32;
    eprintln!("\nTile 344 first 16 bytes: {:02X?}", &vram[off..off + 16]);
    eprintln!(
        "Tile 394 first 16 bytes: {:02X?}",
        &vram[394 * 32..394 * 32 + 16]
    );

    // Check if the tiles contain the WRONG data (not zero but wrong palette/indices)
    let mut all_zero_count = 0;
    let mut nonzero_but_wrong = 0;
    for tid in 344..=472u32 {
        let off = tid as usize * 32;
        let is_all_zero = vram[off..off + 32].iter().all(|&b| b == 0);
        if is_all_zero {
            all_zero_count += 1;
        }
    }
    eprintln!(
        "All-zero: {}/129, has-data: {}/129",
        all_zero_count,
        129 - all_zero_count
    );
}

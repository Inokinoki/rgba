use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.palette_log_enabled = true;

    for frame in 0..500u32 {
        let log_start = gba.mem.palette_write_log.len();

        gba.run_frame_parallel(&mut fb);

        // Find OBJ palette writes this frame
        let obj_writes: Vec<_> = gba.mem.palette_write_log[log_start..]
            .iter()
            .filter(|(addr, _, _, _)| *addr >= 0x05000200 && *addr < 0x05000400)
            .collect();

        // Find zero writes to OBJ palette
        let zero_writes: Vec<_> = obj_writes
            .iter()
            .filter(|(_, _, val, _)| *val == 0)
            .collect();
        let nonzero_writes: Vec<_> = obj_writes
            .iter()
            .filter(|(_, _, val, _)| *val != 0)
            .collect();

        if !obj_writes.is_empty() {
            println!(
                "Frame {}: {} OBJ writes ({} zero, {} nonzero)",
                frame,
                obj_writes.len(),
                zero_writes.len(),
                nonzero_writes.len()
            );

            // Show first few non-zero writes
            for (addr, pc, val, dma) in nonzero_writes.iter().take(5) {
                println!(
                    "  NZ: addr={:08X} val={:02X} pc={:08X} dma={}",
                    addr, val, pc, dma
                );
            }
            // Show first few zero writes
            for (addr, pc, val, dma) in zero_writes.iter().take(5) {
                println!(
                    "  Z:  addr={:08X} val={:02X} pc={:08X} dma={}",
                    addr, val, pc, dma
                );
            }
        }

        // Check OBJ palette state
        let pal = gba.mem.palette();
        let mut obj_nonzero = 0u32;
        for i in 0..256 {
            let off = 0x200 + i * 2;
            let c = u16::from_le_bytes([pal[off], pal[off + 1]]);
            if c != 0 {
                obj_nonzero += 1;
            }
        }

        if frame >= 190 && frame <= 200 {
            println!("  -> Frame {} end: OBJ nonzero={}", frame, obj_nonzero);
        }
    }
}

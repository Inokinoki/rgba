use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.palette_log_enabled = true;

    let mut last_obj_nonzero_frame = 0u32;

    for frame in 0..500u32 {
        let log_start = gba.mem.palette_write_log.len();

        gba.run_frame_parallel(&mut fb);

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

        if obj_nonzero > 0 {
            last_obj_nonzero_frame = frame;
        }

        // Find OBJ palette writes in this frame
        let obj_writes: Vec<_> = gba.mem.palette_write_log[log_start..]
            .iter()
            .filter(|(addr, _, _, _)| *addr >= 0x05000200 && *addr < 0x05000400)
            .collect();

        if !obj_writes.is_empty() || (frame >= 190 && frame <= 210) {
            println!(
                "Frame {}: obj_nonzero={} obj_writes={}",
                frame,
                obj_nonzero,
                obj_writes.len()
            );
        }

        // If OBJ palette went from non-zero to zero, that's important
        if frame > 192 && obj_nonzero == 0 && last_obj_nonzero_frame == frame - 1 {
            println!(
                "!!! OBJ palette zeroed between frame {} and {}!",
                frame - 1,
                frame
            );
            // Show all palette writes this frame
            let all_writes: Vec<_> = gba.mem.palette_write_log[log_start..]
                .iter()
                .filter(|(addr, _, _, _)| *addr >= 0x05000200 && *addr < 0x05000400)
                .collect();
            for (addr, pc, val, dma) in all_writes.iter().take(50) {
                println!(
                    "  ZERO: addr={:08X} val={:02X} pc={:08X} dma={}",
                    addr, val, pc, dma
                );
            }
        }
    }

    println!(
        "\nLast frame with non-zero OBJ palette: {}",
        last_obj_nonzero_frame
    );
}

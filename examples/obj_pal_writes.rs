use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.palette_log_enabled = true;

    for frame in 0..5u32 {
        let before = gba.mem.palette_write_log.len();
        gba.run_frame_parallel(&mut fb);
        let new_writes = &gba.mem.palette_write_log[before..];

        let obj_writes: Vec<_> = new_writes
            .iter()
            .filter(|(addr, _, _, _)| *addr >= 0x05000200 && *addr < 0x05000400)
            .collect();

        if !obj_writes.is_empty() {
            println!(
                "Frame {}: {} OBJ palette writes (total palette writes: {})",
                frame,
                obj_writes.len(),
                new_writes.len()
            );
            // Show first 20 OBJ writes
            for (addr, pc, val, dma) in obj_writes.iter().take(20) {
                println!("  {:08X}={:02X} PC={:08X} dma={}", addr, val, pc << 1, dma);
            }
        }
    }
}

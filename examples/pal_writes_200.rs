use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.palette_log_enabled = true;

    for frame in 0..200u32 {
        let before = gba.mem.palette_write_log.len();
        gba.run_frame_parallel(&mut fb);
        let new_writes = &gba.mem.palette_write_log[before..];

        let obj_writes: Vec<_> = new_writes
            .iter()
            .filter(|(addr, _, _, _)| *addr >= 0x05000200)
            .collect();

        if !obj_writes.is_empty() {
            println!(
                "Frame {:4}: {} OBJ+ palette writes",
                frame,
                obj_writes.len()
            );
            for (addr, pc, val, dma) in obj_writes.iter().take(10) {
                println!("  {:08X}={:02X} PC={:08X} dma={}", addr, val, pc << 1, dma);
            }
            if obj_writes.len() > 10 {
                println!("  ... and {} more", obj_writes.len() - 10);
            }
        }
    }

    // Also check: total OBJ palette writes across all frames
    let obj_total = gba
        .mem
        .palette_write_log
        .iter()
        .filter(|(addr, _, _, _)| *addr >= 0x05000200)
        .count();
    println!("\nTotal OBJ+ palette writes in 200 frames: {}", obj_total);

    // Check BG palette writes too
    let bg_total = gba
        .mem
        .palette_write_log
        .iter()
        .filter(|(addr, _, _, _)| *addr >= 0x05000000 && *addr < 0x05000200)
        .count();
    println!("Total BG palette writes in 200 frames: {}", bg_total);
}

use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.palette_log_enabled = true;

    for frame in 0..193u32 {
        gba.run_frame_parallel(&mut fb);
    }

    // Now at frame 192 complete. Log all OBJ palette writes in this frame
    // We need to capture just frame 192's writes
    let before = gba.mem.palette_write_log.len();
    gba.run_frame_parallel(&mut fb); // frame 192

    let obj_writes: Vec<_> = gba.mem.palette_write_log[before..]
        .iter()
        .filter(|(addr, _, _, _)| *addr >= 0x05000200)
        .collect();

    println!("Frame 192: {} OBJ palette writes", obj_writes.len());
    for (addr, pc, val, dma) in obj_writes.iter() {
        println!("  {:08X}={:02X} PC={:08X} dma={}", addr, val, pc << 1, dma);
    }

    // Also show BG palette writes
    let bg_writes: Vec<_> = gba.mem.palette_write_log[before..]
        .iter()
        .filter(|(addr, _, _, _)| *addr >= 0x05000000 && *addr < 0x05000200)
        .collect();
    println!("\nFrame 192: {} BG palette writes", bg_writes.len());
    for (addr, pc, val, dma) in bg_writes.iter().take(20) {
        println!("  {:08X}={:02X} PC={:08X} dma={}", addr, val, pc << 1, dma);
    }

    // Check OBJ palette content
    println!("\nOBJ palette non-zero entries:");
    let pal = gba.mem.palette();
    for i in 128..256 {
        let v = u16::from_le_bytes([pal[i * 2], pal[i * 2 + 1]]);
        if v != 0 {
            println!("  PAL[{}]={:04X}", i, v);
        }
    }
}

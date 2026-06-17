use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.ewram_range_log_enabled = true;

    for frame in 0..5u32 {
        let log_start = gba.mem.ewram_range_log.len();

        gba.run_frame_parallel(&mut fb);

        let new_writes = &gba.mem.ewram_range_log[log_start..];
        if !new_writes.is_empty() {
            println!(
                "Frame {}: {} writes to EWRAM 0x8000-0x8A00",
                frame,
                new_writes.len()
            );
            for (addr, pc, val) in new_writes.iter() {
                let nz = if *val != 0 { "NZ" } else { "z " };
                println!("  {:08X} = {:02X} PC={:08X} {}", addr, val, pc, nz);
            }
        }
    }
}

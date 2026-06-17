use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    gba.mem.timer_writes_enabled = true;
    gba.mem.timer_writes.clear();

    println!("=== Tracing writes to 0x02000050 for 5 frames ===");
    for f in 0..5 {
        let v50_before = gba.mem.read_word(0x02000050);
        let write_count_before = gba.mem.timer_writes.len();
        gba.run_frame_parallel(&mut fb);
        let v50_after = gba.mem.read_word(0x02000050);
        let write_count_after = gba.mem.timer_writes.len();
        println!(
            "Frame {}: [50]={:08X}->{:08X}  new_writes={}",
            200 + f,
            v50_before,
            v50_after,
            write_count_after - write_count_before
        );
        for i in write_count_before..write_count_after {
            let (addr, pc, val) = gba.mem.timer_writes[i];
            println!("  WRITE addr={:08X} pc={:08X} val={:02X}", addr, pc, val);
        }
    }
}

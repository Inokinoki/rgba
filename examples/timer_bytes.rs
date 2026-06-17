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

    println!("=== Monitoring [r5+0..+2] at 0x02000050 ===");
    for f in 0..30 {
        let byte0 = gba.mem.read_byte(0x02000050) as u32;
        let byte1 = gba.mem.read_byte(0x02000051) as u32;
        let word = gba.mem.read_word(0x02000050);
        println!(
            "Frame {}: byte0={:02X} byte1={:02X} word={:08X}",
            200 + f,
            byte0,
            byte1,
            word
        );
        gba.run_frame_parallel(&mut fb);
    }
}

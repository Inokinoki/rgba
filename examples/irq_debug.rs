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

    for f in 0..10 {
        let v50 = gba.mem.read_word(0x02000050);
        let v74 = gba.mem.read_word(0x02000074);
        println!("Frame {}: [50]={:08X} [74]={:08X}", 200+f, v50, v74);
        gba.run_frame_parallel(&mut fb);
    }
}

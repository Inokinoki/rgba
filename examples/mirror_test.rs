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

    // Read IWRAM at 0x03007FFC
    let val_direct = gba.mem.read_word(0x03007FFC);
    println!("0x03007FFC: {:08X} (direct)", val_direct);

    // Read via mirror at 0x03FFFFFC
    let val_mirror = gba.mem.read_word(0x03FFFFFC);
    println!("0x03FFFFFC: {:08X} (mirror)", val_mirror);
}

use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..3 { gba.run_frame_parallel(&mut fb); }

    let m = gba.mem_mut();

    // Dump THUMB code at 0x080D2F00-0x080D2F40 (main loop area)
    let mut data = Vec::new();
    for i in 0..48 {
        let addr = 0x080D2F00 + i * 2;
        let half = m.read_half(addr);
        data.extend_from_slice(&half.to_le_bytes());
    }
    std::fs::write("/tmp/mainloop.bin", &data).unwrap();
    println!("Wrote main loop code to /tmp/mainloop.bin");

    // Also check what [0x03007FF8] is (the handler writes to it)
    println!("[0x03007FF8] = 0x{:04X}", m.read_half(0x03007FF8));
    println!("[0x03007FFA] = 0x{:04X}", m.read_half(0x03007FFA));

    // Check what's at the callback address
    let callback = m.read_word(0x03000450);
    println!("VBlank callback: 0x{:08X}", callback);

    // Check game state variables
    println!("[0x02000074] = 0x{:08X} (game state)", m.read_word(0x02000074));
    println!("[0x03000000] = 0x{:08X}", m.read_word(0x03000000));
    println!("[0x03000004] = 0x{:08X}", m.read_word(0x03000004));
}

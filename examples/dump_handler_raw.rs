use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..3 { gba.run_frame_parallel(&mut fb); }

    let m = gba.mem_mut();
    // Dump handler as raw bytes
    let mut data = Vec::new();
    for i in 0..64 {
        let addr = 0x03000958 + i * 4;
        let word = m.read_word(addr);
        data.extend_from_slice(&word.to_le_bytes());
    }
    std::fs::write("/tmp/handler.bin", &data).unwrap();
    println!("Wrote {} bytes to /tmp/handler.bin", data.len());

    // Also dump BIOS stub
    let mut bios_data = Vec::new();
    for i in 0..24 {
        let word = m.bios_read_word(0x18 + i * 4);
        bios_data.extend_from_slice(&word.to_le_bytes());
    }
    std::fs::write("/tmp/bios_stub.bin", &bios_data).unwrap();
    println!("Wrote BIOS stub to /tmp/bios_stub.bin");
}

use rgba::Gba;

#[test]
#[ignore]
fn trace_harvest_moon() {
    let rom_path = std::env::var("RGBA_ROM_PATH")
        .expect("Set RGBA_ROM_PATH to run this test");
    let rom_data = std::fs::read(&rom_path).expect("Failed to read ROM");

    let mut gba = Gba::new();
    gba.load_rom(rom_data);
    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().dma_log_enabled = true;

    for frame in 0..5u32 {
        gba.run_frame();

        let vram = gba.mem().vram();
        let mut ffff = 0u32;
        for i in 0..1024 {
            let off = 0xC000 + i * 2;
            let e = u16::from_le_bytes([vram[off], vram[off + 1]]);
            if e == 0xFFFF {
                ffff += 1;
            }
        }
        eprintln!(
            "Frame {}: 0xFFFF={} PC=0x{:08X}",
            frame,
            ffff,
            gba.cpu_pc()
        );
    }
}

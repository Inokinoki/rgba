use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..199 {
        gba.run_frame_parallel(&mut fb);
    }

    gba.mem.enable_io_trace(true);

    gba.run_scanline();
    let io = gba.mem.io();
    let bldy = u16::from_le_bytes([io[0x54], io[0x55]]);
    let bldcnt = u16::from_le_bytes([io[0x50], io[0x51]]);
    println!("Scanline 0: BLDY={}, BLDCNT=0x{:04X}", bldy, bldcnt);
    gba.mem.clear_io_trace();

    for sl in 1..161 {
        gba.run_scanline();
        let io = gba.mem.io();
        let bldy = u16::from_le_bytes([io[0x54], io[0x55]]);
        let bldcnt = u16::from_le_bytes([io[0x50], io[0x51]]);

        let writes = gba.mem.get_io_trace();
        let bldy_writes: Vec<_> = writes
            .iter()
            .filter(|(addr, _)| *addr == 0x54 || *addr == 0x55)
            .collect();
        let bldcnt_writes: Vec<_> = writes
            .iter()
            .filter(|(addr, _)| *addr == 0x50 || *addr == 0x51)
            .collect();

        if !bldy_writes.is_empty() || !bldcnt_writes.is_empty() {
            println!(
                "Scanline {:3}: BLDY={}, BLDCNT=0x{:04X} writes_bldy={:?} writes_bldcnt={:?}",
                sl, bldy, bldcnt, bldy_writes, bldcnt_writes
            );
        }
        gba.mem.clear_io_trace();
    }

    gba.mem.enable_io_trace(false);
    let io = gba.mem.io();
    let bldy = u16::from_le_bytes([io[0x54], io[0x55]]);
    println!("\nFinal BLDY: {}", bldy);
}

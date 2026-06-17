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

    println!("=== Frame 200: BLDY per scanline ===");

    for sl in 0..228 {
        gba.run_scanline();
        gba.sync_ppu();
        let bldy = gba.ppu.get_blend_brightness();
        let bldcnt = gba.ppu.get_blend_control();
        if sl <= 5 || (sl >= 158 && sl <= 162) || bldy != 13 || bldcnt != 0x00BF {
            println!(
                "  Scanline {:3}: BLDY={} BLDCNT=0x{:04X} mode={}",
                sl,
                bldy,
                bldcnt,
                (bldcnt >> 6) & 3
            );
        }
    }

    let bldy = gba.ppu.get_blend_brightness();
    let bldcnt = gba.ppu.get_blend_control();
    println!(
        "\nFinal: BLDY={} BLDCNT=0x{:04X} mode={}",
        bldy,
        bldcnt,
        (bldcnt >> 6) & 3
    );
}

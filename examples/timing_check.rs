use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..462 {
        for _ in 0..228 {
            gba.run_scanline();
        }
    }

    // Now run scanlines 0-159 (visible area) of the NEXT frame and sync
    for scanline in 0..228 {
        gba.run_scanline();

        if scanline == 159 || scanline == 160 || scanline == 161 || scanline == 227 {
            gba.sync_ppu_full();
            let dispcnt = gba.ppu.get_dispcnt();
            let bg0cnt = gba.ppu.get_bgcnt(0);
            let bg3cnt = gba.ppu.get_bgcnt(3);
            println!(
                "Scanline {:3}: DISPCNT=0x{:04X} BG0CNT=0x{:04X} BG3CNT=0x{:04X}",
                scanline, dispcnt, bg0cnt, bg3cnt
            );
        }
    }
}

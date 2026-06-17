use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    let mut last_dispcnt = 0u16;
    for frame in 0..1200 {
        gba.run_frame_parallel(&mut fb);
        let dispcnt = gba.ppu.get_dispcnt();
        let bg0cnt = gba.ppu.get_bgcnt(0);
        if dispcnt != last_dispcnt || frame % 200 == 0 {
            let mode = dispcnt & 7;
            let bg_en = (dispcnt >> 8) & 0xF;
            let obj_en = (dispcnt >> 12) & 1;
            println!(
                "Frame {:4}: DISPCNT=0x{:04X} mode={} bg={:04b} obj={} BG0CNT=0x{:04X}",
                frame, dispcnt, mode, bg_en, obj_en, bg0cnt
            );
            last_dispcnt = dispcnt;
        }
    }
}

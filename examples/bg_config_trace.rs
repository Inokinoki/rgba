use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..300 {
        gba.run_frame_parallel(&mut fb);

        if frame % 5 == 0 {
            gba.sync_ppu_full();
            let ppu = gba.ppu();
            for bg in 0..4 {
                let bgcnt = ppu.get_bgcnt(bg);
                let enabled = ppu.is_bg_enabled(bg);
                if enabled {
                    let char_base = ((bgcnt >> 2) & 3) * 0x4000;
                    let screen_base = ((bgcnt >> 8) & 0x1F) * 0x800;
                    if char_base != 0 || screen_base != 0 {
                        println!(
                            "Frame {:3} BG{}: cnt={:#06X} char={:#X} screen={:#X} pri={}",
                            frame,
                            bg,
                            bgcnt,
                            char_base,
                            screen_base,
                            bgcnt & 3
                        );
                    }
                }
            }
        }
    }
}

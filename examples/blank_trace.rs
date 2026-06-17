use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..600 {
        gba.run_frame_parallel(&mut fb);

        let ppu = &gba.ppu;
        let dispcnt = ppu.get_dispcnt();
        let forced_blank = (dispcnt & 0x80) != 0;
        let hofs = ppu.get_bg_hofs(0);
        let bldy = ppu.get_blend_brightness();

        let unique: std::collections::HashSet<u32> = fb.iter().copied().collect();

        if !forced_blank || frame <= 5 || (frame >= 188 && frame <= 200) {
            println!(
                "Frame {:3}: dispcnt=0x{:04X} blank={} hofs={} BLDY={} unique_colors={}",
                frame,
                dispcnt,
                forced_blank,
                hofs,
                bldy,
                unique.len()
            );
        }
    }
}

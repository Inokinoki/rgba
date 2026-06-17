use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    println!("=== BG0 hofs/vofs trace over frames ===");
    for frame in 0..600 {
        gba.run_frame_parallel(&mut fb);
        let ppu = &gba.ppu;
        let hofs = ppu.get_bg_hofs(0);
        let vofs = ppu.get_bg_vofs(0);
        let bldy = ppu.get_blend_brightness();
        if frame < 10 || frame % 50 == 0 || (frame >= 190 && frame <= 210) || hofs != 224 {
            println!(
                "Frame {:3}: BG0 hofs={}, vofs={}, BLDY={}",
                frame, hofs, vofs, bldy
            );
        }
    }
}

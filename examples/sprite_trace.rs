use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..1200 {
        gba.run_frame_parallel(&mut fb);

        let ppu = &gba.ppu;
        let mut sprite_count = 0;
        for s in 0..128 {
            if ppu.sprite_is_enabled(s) && !ppu.sprite_is_window(s) {
                sprite_count += 1;
            }
        }

        if sprite_count > 0 || frame % 100 == 0 {
            println!("Frame {:4}: sprites={}", frame, sprite_count);
        }
    }
}

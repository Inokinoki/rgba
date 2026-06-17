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
        let mut on_screen = 0;
        let mut off_screen = 0;
        let mut first_3 = Vec::new();
        for s in 0..128usize {
            if ppu.sprite_is_enabled(s) && !ppu.sprite_is_window(s) {
                let y = ppu.sprite_y(s);
                let x = ppu.sprite_x(s);
                let (w, h) = ppu.sprite_dimensions(s);
                if y < 160 && x < 240 && y + h as i32 > 0 {
                    on_screen += 1;
                    if first_3.len() < 3 {
                        first_3.push(format!("S{}:x={},y={},{}x{}", s, x, y, w, h));
                    }
                } else {
                    off_screen += 1;
                }
            }
        }

        if on_screen > 0 || frame % 200 == 0 {
            println!(
                "Frame {:4}: on_screen={} off_screen={} [{}]",
                frame,
                on_screen,
                off_screen,
                first_3.join(", ")
            );
        }
    }
}

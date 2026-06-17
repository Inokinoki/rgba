use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..481 {
        gba.run_frame_parallel(&mut fb);

        if frame == 479 || frame == 480 {
            let ppu = &gba.ppu;
            let mut count = 0;
            for s in 0..128usize {
                if ppu.sprite_is_enabled(s) {
                    count += 1;
                    if count <= 3 {
                        let y = ppu.sprite_y(s);
                        let x = ppu.sprite_x(s);
                        let (w, h) = ppu.sprite_dimensions(s);
                        let tile = ppu.sprite_tile(s);
                        println!(
                            "  Frame {} S{:3}: x={} y={} {}x{} tile={}",
                            frame, s, x, y, w, h, tile
                        );
                    }
                }
            }
            println!("Frame {}: total_enabled={}", frame, count);
        }
    }
}

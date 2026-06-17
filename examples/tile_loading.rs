use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..600 {
        gba.run_frame_parallel(&mut fb);

        if frame % 50 == 0 {
            gba.sync_ppu_full();
            let vram = gba.ppu().vram();
            let bg0_tiles = [394, 403, 412, 420, 473, 482, 491, 499];
            let mut loaded = 0;
            for &t in &bg0_tiles {
                if t < 512 {
                    let off = t * 32;
                    let mut nonzero = 0;
                    for b in 0..32 {
                        if vram[off + b] != 0 {
                            nonzero += 1;
                        }
                    }
                    if nonzero > 0 {
                        loaded += 1;
                    }
                }
            }
            println!("Frame {}: {}/8 BG0 tiles loaded", frame, loaded);
        }
    }
}

use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for frame in 0..200 {
        for _scanline in 0..228 {
            gba.run_scanline();
        }

        // Check ALL char blocks (0-3) for nonzero content
        let vram = gba.mem().vram();
        if frame == 0 || frame == 5 || frame == 6 || frame == 189 || frame == 190 || frame == 199 {
            for cb in 0..4 {
                let base = cb * 0x4000;
                let mut nonzero = 0;
                for tile in 0..512 {
                    let off = base + tile * 32;
                    if off + 32 <= vram.len() {
                        if vram[off..off + 32].iter().any(|&b| b != 0) {
                            nonzero += 1;
                        }
                    }
                }
                if nonzero > 0 {
                    println!("Frame {:3} CB{}: {} nonzero tiles", frame, cb, nonzero);
                }
            }
        }
    }
}

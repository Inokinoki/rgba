use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for frame in 0..200 {
        for _ in 0..228 {
            gba.run_scanline();
        }

        let vram = gba.mem().vram();
        let mut nonzero_scr = 0;
        for i in 0..1024 {
            let off = 0xC000 + i * 2;
            let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
            if entry != 0 {
                nonzero_scr += 1;
            }
        }

        let mut nonzero_tiles = 0;
        for tile in 96..512 {
            let off = tile * 32;
            if vram[off..off + 32].iter().any(|&b| b != 0) {
                nonzero_tiles += 1;
            }
        }

        if nonzero_scr > 0 || nonzero_tiles > 0 {
            println!(
                "Frame {:3}: BG0SCR nonzero={:4} tiles(96+)={}",
                frame, nonzero_scr, nonzero_tiles
            );
        }
    }
}

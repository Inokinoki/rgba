use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for frame in 0..1200 {
        for _ in 0..228 {
            gba.run_scanline();
        }

        if frame % 200 == 0 || frame == 300 || frame == 400 || frame == 500 {
            let io = gba.mem().io();
            let dispcnt = u16::from_le_bytes([io[0], io[1]]);
            let vcount = gba.ppu().get_vcount();

            let vram = gba.mem().vram();
            let mut nonzero = 0;
            for tile in 96..512 {
                let off = tile * 32;
                if vram[off..off + 32].iter().any(|&b| b != 0) {
                    nonzero += 1;
                }
            }

            println!(
                "Frame {:4}: DISPCNT={:04X} tiles96-511={}",
                frame, dispcnt, nonzero
            );
        }
    }
}

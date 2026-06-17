use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let io = gba.mem().io();
    let dispcnt = u16::from_le_bytes([io[0], io[1]]);
    println!("Initial DISPCNT: {:#06X}", dispcnt);

    let mut fb = vec![0u32; 240 * 160];

    let mut vram_clear_count = 0;
    let mut last_vram_clear_frame = 0;

    for frame in 0..400 {
        let before = {
            let vram = gba.mem().vram();
            let mut nonzero = 0;
            for b in vram.iter() {
                if *b != 0 {
                    nonzero += 1;
                }
            }
            nonzero
        };

        gba.run_frame_parallel(&mut fb);

        let after = {
            let vram = gba.mem().vram();
            let mut nonzero = 0;
            for b in vram.iter() {
                if *b != 0 {
                    nonzero += 1;
                }
            }
            nonzero
        };

        if after < before - 100 {
            vram_clear_count += 1;
            last_vram_clear_frame = frame;
            if vram_clear_count <= 5 {
                println!(
                    "Frame {}: VRAM dropped from {} to {} nonzero bytes",
                    frame, before, after
                );
            }
        }
    }

    println!("\nTotal VRAM clear events: {}", vram_clear_count);
    println!("Last clear at frame: {}", last_vram_clear_frame);
}

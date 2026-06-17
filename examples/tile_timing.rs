use rgba::Gba;

fn count_tiles_with_data(vram: &[u8]) -> (usize, u16) {
    let mut count = 0;
    let mut last = 0u16;
    for t in 0..512u16 {
        let start = t as usize * 32;
        let mut has = false;
        for b in 0..32 {
            if vram[start + b] != 0 {
                has = true;
                break;
            }
        }
        if has {
            count += 1;
            last = t;
        }
    }
    (count, last)
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let checkpoints = [100, 500, 1000, 2000, 5000, 10000, 20000, 50000];
    let mut next_ci = 0;
    let mut frame = 0u32;

    loop {
        if next_ci < checkpoints.len() && frame >= checkpoints[next_ci] {
            gba.sync_ppu_full();
            let (count, last) = count_tiles_with_data(gba.ppu().vram());
            println!("Frame {}: {} tiles with data, last={}", frame, count, last);
            next_ci += 1;
        }
        if next_ci >= checkpoints.len() {
            break;
        }
        gba.run_frame();
        frame += 1;
    }
}

use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }

    gba.input.press_key(KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input.release_key(KeyState::START);
    for _ in 0..180 {
        gba.run_frame_parallel(&mut fb);
    }

    for round in 0..500 {
        gba.input.press_key(KeyState::A);
        for _ in 0..4 {
            gba.run_frame_parallel(&mut fb);
        }
        gba.input.release_key(KeyState::A);
        for _ in 0..16 {
            gba.run_frame_parallel(&mut fb);
        }

        if [79, 150, 200, 250, 300, 350, 400, 499].contains(&round) {
            let path = format!("/tmp/gameplay_r{}.ppm", round);
            save_ppm(&fb, &path);
            println!("Saved {}", path);
        }
    }
}

fn save_ppm(fb: &[u32], path: &str) {
    let mut out = format!("P6\n240 160\n255\n");
    let header = out.as_bytes().to_vec();
    let mut data = Vec::with_capacity(header.len() + 240 * 160 * 3);
    data.extend_from_slice(&header);
    for i in 0..240 * 160 {
        let p = fb[i];
        data.push((p & 0xFF) as u8);
        data.push(((p >> 8) & 0xFF) as u8);
        data.push(((p >> 16) & 0xFF) as u8);
    }
    std::fs::write(path, &data).unwrap();
}

use rgba::Gba;
use rgba::KeyState;

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

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Boot through title
    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }

    // Press START to get past title
    gba.input.press_key(KeyState::START);
    for _ in 0..4 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input.release_key(KeyState::START);
    for _ in 0..180 {
        gba.run_frame_parallel(&mut fb);
    }

    // Press through menus with A - enough to reach farm gameplay
    for _ in 0..80 {
        gba.input.press_key(KeyState::A);
        for _ in 0..4 {
            gba.run_frame_parallel(&mut fb);
        }
        gba.input.release_key(KeyState::A);
        for _ in 0..16 {
            gba.run_frame_parallel(&mut fb);
        }
    }

    // Extra frames to settle
    for _ in 0..120 {
        gba.run_frame_parallel(&mut fb);
    }

    save_ppm(&fb, "/tmp/farm_scene.ppm");
    println!("Saved farm_scene.ppm");

    let mut colors = std::collections::HashMap::new();
    for &p in &fb {
        *colors.entry(p).or_insert(0u32) += 1;
    }
    println!("Farm scene: {} unique colors", colors.len());
    for (&c, &n) in colors.iter() {
        let pct = n as f64 / (240.0 * 160.0) * 100.0;
        if pct > 2.0 {
            println!("  color {:#010X}: {:.1}%", c, pct);
        }
    }
}

use rgba::{Gba, KeyState};

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    let mut ppm = String::new();

    // Boot
    for _ in 0..500 {
        gba.run_frame_parallel(&mut fb);
    }

    // START to enter menu
    gba.input_mut().press_key(KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut fb);
    }

    // A to select
    gba.input_mut().press_key(KeyState::A);
    for _ in 0..30 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(KeyState::A);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut fb);
    }

    // Try navigating: A again
    for round in 0..6 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..15 {
            gba.run_frame_parallel(&mut fb);
        }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..60 {
            gba.run_frame_parallel(&mut fb);
        }

        ppm.clear();
        ppm.push_str("P3\n240 160\n255\n");
        for y in 0..160 {
            for x in 0..240 {
                let c = fb[y * 240 + x];
                ppm.push_str(&format!(
                    "{} {} {} ",
                    (c >> 16) & 0xFF,
                    (c >> 8) & 0xFF,
                    c & 0xFF
                ));
            }
            ppm.push('\n');
        }
        let path = format!("/tmp/nav_a{}.ppm", round);
        std::fs::write(&path, &ppm).unwrap();

        let unique: std::collections::HashSet<u32> = fb.iter().copied().collect();
        println!("Round {}: {} unique colors", round, unique.len());
    }
}

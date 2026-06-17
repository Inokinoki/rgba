use rgba::{Gba, KeyState};

fn save_frame(gba: &mut Gba, fb: &mut [u32], n: usize, path: &str) {
    for _ in 0..n {
        gba.run_frame_parallel(fb);
    }
    let mut ppm = String::from("P3\n240 160\n255\n");
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
    std::fs::write(path, ppm).unwrap();
    println!("Saved {}", path);
}

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Boot to title
    save_frame(&mut gba, &mut fb, 500, "/tmp/pt_00_title.ppm");

    // Hold START for 60 frames
    gba.input_mut().press_key(KeyState::START);
    save_frame(&mut gba, &mut fb, 60, "/tmp/pt_01_start_held.ppm");
    gba.input_mut().release_key(KeyState::START);

    // Wait and see what happens
    save_frame(&mut gba, &mut fb, 120, "/tmp/pt_02_after_start.ppm");

    // Try pressing A for 60 frames
    gba.input_mut().press_key(KeyState::A);
    save_frame(&mut gba, &mut fb, 60, "/tmp/pt_03_a_held.ppm");
    gba.input_mut().release_key(KeyState::A);

    save_frame(&mut gba, &mut fb, 120, "/tmp/pt_04_after_a.ppm");
}

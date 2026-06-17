use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Frame 200: title screen
    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }
    // Convert framebuffer to PPM then PNG
    let mut ppm = String::from("P3\n240 160\n255\n");
    for y in 0..160 {
        for x in 0..240 {
            let c = fb[y * 240 + x];
            let r = (c >> 16) & 0xFF;
            let g = (c >> 8) & 0xFF;
            let b = c & 0xFF;
            ppm.push_str(&format!("{} {} {} ", r, g, b));
        }
        ppm.push('\n');
    }
    std::fs::write("/tmp/title_200.ppm", ppm).unwrap();

    // Frame 568: just before state change
    for _ in 0..368 {
        gba.run_frame_parallel(&mut fb);
    }
    let mut ppm2 = String::from("P3\n240 160\n255\n");
    for y in 0..160 {
        for x in 0..240 {
            let c = fb[y * 240 + x];
            let r = (c >> 16) & 0xFF;
            let g = (c >> 8) & 0xFF;
            let b = c & 0xFF;
            ppm2.push_str(&format!("{} {} {} ", r, g, b));
        }
        ppm2.push('\n');
    }
    std::fs::write("/tmp/title_568.ppm", ppm2).unwrap();

    // Frame 700: demo mode
    for _ in 0..132 {
        gba.run_frame_parallel(&mut fb);
    }
    let mut ppm3 = String::from("P3\n240 160\n255\n");
    for y in 0..160 {
        for x in 0..240 {
            let c = fb[y * 240 + x];
            let r = (c >> 16) & 0xFF;
            let g = (c >> 8) & 0xFF;
            let b = c & 0xFF;
            ppm3.push_str(&format!("{} {} {} ", r, g, b));
        }
        ppm3.push('\n');
    }
    std::fs::write("/tmp/demo_700.ppm", ppm3).unwrap();

    println!("Screenshots saved to /tmp/title_200.ppm, /tmp/title_568.ppm, /tmp/demo_700.ppm");
}

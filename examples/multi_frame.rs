use rgba::Gba;

fn save_frame(gba: &mut Gba, fb: &mut Vec<u32>, target: usize, path: &str) {
    fb.fill(0);
    for _ in 0..target {
        gba.run_frame_parallel(fb);
    }
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
    std::fs::write(path, ppm).unwrap();
    println!("Saved {}", path);
}

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    save_frame(&mut gba, &mut fb, 480, "/tmp/f480_sprites.ppm");
    save_frame(&mut gba, &mut fb, 490, "/tmp/f490_sprites.ppm");

    let mut gba2 = Gba::new();
    gba2.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba2.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb2 = vec![0u32; 240 * 160];
    save_frame(&mut gba2, &mut fb2, 770, "/tmp/f770_attract.ppm");
    save_frame(&mut gba2, &mut fb2, 970, "/tmp/f970_attract.ppm");
}

use rgba::Gba;
use std::fs;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..500u32 {
        gba.run_frame_parallel(&mut fb);
    }

    // Save PPM
    let mut ppm = String::from("P6\n240 160\n255\n");
    let mut bytes = ppm.into_bytes();
    for y in 0..160 {
        for x in 0..240 {
            let pixel = fb[y * 240 + x];
            let r = ((pixel >> 16) & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let b = (pixel & 0xFF) as u8;
            bytes.push(r);
            bytes.push(g);
            bytes.push(b);
        }
    }
    fs::write("/tmp/frame500.ppm", &bytes).unwrap();

    // Convert to PNG
    println!("Saved frame 500 to /tmp/frame500.ppm");
}

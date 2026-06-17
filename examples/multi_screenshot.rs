use rgba::Gba;
use std::fs;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Save frames at key points
    for frame in 0..=200u32 {
        gba.run_frame_parallel(&mut fb);

        if [1, 5, 10, 20, 50, 100, 150, 192, 200].contains(&frame) {
            let mut ppm = String::from("P6\n240 160\n255\n");
            let mut bytes = ppm.into_bytes();
            for y in 0..160 {
                for x in 0..240 {
                    let pixel = fb[y * 240 + x];
                    bytes.push(((pixel >> 16) & 0xFF) as u8);
                    bytes.push(((pixel >> 8) & 0xFF) as u8);
                    bytes.push((pixel & 0xFF) as u8);
                }
            }
            let path = format!("/tmp/frame{}.ppm", frame);
            fs::write(&path, &bytes).unwrap();
            println!("Saved {}", path);
        }
    }
}

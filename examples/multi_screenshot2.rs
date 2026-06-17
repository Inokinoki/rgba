use rgba::Gba;
use std::fs;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    let frames = [100, 200, 300, 400, 500, 600, 800, 1000, 1500, 2000];
    let mut next_idx = 0;

    for frame in 0..=2000u32 {
        gba.run_frame_parallel(&mut fb);

        if next_idx < frames.len() && frame == frames[next_idx] {
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
            let path = format!("/tmp/f{}.ppm", frame);
            fs::write(&path, &bytes).unwrap();
            next_idx += 1;
        }
    }

    for f in &frames {
        let png = format!("/tmp/f{}.png", f);
        let ppm = format!("/tmp/f{}.ppm", f);
        python3_convert(&ppm, &png);
    }
}

fn python3_convert(ppm: &str, png: &str) {
    use std::process::Command;
    Command::new("python3")
        .args([
            "-c",
            &format!(
                "from PIL import Image; Image.open('{}').save('{}')",
                ppm, png
            ),
        ])
        .output()
        .unwrap();
}

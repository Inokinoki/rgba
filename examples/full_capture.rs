use rgba::Gba;
use std::fs;
use std::process::Command;

fn save_ppm_png(fb: &[u32], path: &str) {
    let ppm = path.replace(".png", ".ppm");
    let mut bytes = b"P6\n240 160\n255\n".to_vec();
    for y in 0..160 {
        for x in 0..240 {
            let p = fb[y * 240 + x];
            bytes.push(((p >> 16) & 0xFF) as u8);
            bytes.push(((p >> 8) & 0xFF) as u8);
            bytes.push((p & 0xFF) as u8);
        }
    }
    fs::write(&ppm, &bytes).unwrap();
    Command::new("python3")
        .args([
            "-c",
            &format!(
                "from PIL import Image; Image.open('{}').save('{}')",
                ppm, path
            ),
        ])
        .output()
        .unwrap();
}

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Run to various frames and save
    let mut frame = 0u32;
    let targets = [300, 500, 800];
    for t in &targets {
        while frame < *t {
            gba.run_frame_parallel(&mut fb);
            frame += 1;
        }
        save_ppm_png(&fb, &format!("/tmp/title_f{}.png", t));
        println!("Saved title_f{}.png", t);
    }

    // Press START to get past title
    gba.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..10 {
        gba.run_frame_parallel(&mut fb);
        frame += 1;
    }
    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut fb);
        frame += 1;
    }
    save_ppm_png(&fb, "/tmp/after_title.png");
    println!("Saved after_title.png (frame {})", frame);

    // Press A to advance dialogue
    for press in 0..5 {
        gba.input_mut().press_key(rgba::KeyState::A);
        for _ in 0..10 {
            gba.run_frame_parallel(&mut fb);
            frame += 1;
        }
        gba.input_mut().release_key(rgba::KeyState::A);
        for _ in 0..120 {
            gba.run_frame_parallel(&mut fb);
            frame += 1;
        }
        save_ppm_png(&fb, &format!("/tmp/dialogue_{}.png", press));
        println!("Saved dialogue_{}.png (frame {})", press, frame);
    }

    // Wait longer for gameplay
    for _ in 0..300 {
        gba.run_frame_parallel(&mut fb);
        frame += 1;
    }
    save_ppm_png(&fb, "/tmp/gameplay.png");
    println!("Saved gameplay.png (frame {})", frame);
}

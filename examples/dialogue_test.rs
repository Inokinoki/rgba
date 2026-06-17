use rgba::Gba;
use std::fs;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Wait for intro, then press START to advance
    for frame in 0..300u32 {
        gba.run_frame_parallel(&mut fb);
    }

    // Press START
    gba.input_mut().press_key(rgba::KeyState::START);
    for frame in 0..60u32 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::START);

    // Wait for transition
    for frame in 0..300u32 {
        gba.run_frame_parallel(&mut fb);
    }

    // Save screenshot
    save_screenshot(&fb, "/tmp/after_start.png");
    println!("Saved after_start.png");

    // Press A to advance dialogue
    gba.input_mut().press_key(rgba::KeyState::A);
    for frame in 0..10u32 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::A);
    for frame in 0..120u32 {
        gba.run_frame_parallel(&mut fb);
    }

    save_screenshot(&fb, "/tmp/after_a1.png");
    println!("Saved after_a1.png");

    // Press A again
    gba.input_mut().press_key(rgba::KeyState::A);
    for frame in 0..10u32 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::A);
    for frame in 0..120u32 {
        gba.run_frame_parallel(&mut fb);
    }

    save_screenshot(&fb, "/tmp/after_a2.png");
    println!("Saved after_a2.png");

    // Try pressing START again for title screen
    for _ in 0..300 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..10 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut fb);
    }

    save_screenshot(&fb, "/tmp/after_start2.png");
    println!("Saved after_start2.png");
}

fn save_screenshot(fb: &[u32], png_path: &str) {
    let ppm_path = png_path.replace(".png", ".ppm");
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
    fs::write(&ppm_path, &bytes).unwrap();
    use std::process::Command;
    Command::new("python3")
        .args([
            "-c",
            &format!(
                "from PIL import Image; Image.open('{}').save('{}')",
                ppm_path, png_path
            ),
        ])
        .output()
        .unwrap();
}

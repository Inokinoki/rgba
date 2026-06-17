use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..250 {
        gba.run_frame_parallel(&mut fb);
    }

    println!("=== Press START+A at frame 250 (well before auto-progression at 569) ===");
    gba.input.press_key(KeyState::START);
    gba.input.press_key(KeyState::A);

    for frame in 250..400 {
        gba.run_frame_parallel(&mut fb);
        let state = gba.mem.read_word(0x02000074);
        let dispcnt = u16::from_le_bytes([gba.mem.io()[0], gba.mem.io()[1]]);
        if frame <= 260 || frame % 10 == 0 || state != 1 {
            println!(
                "Frame {:4}: State={:08X} DISPCNT={:04X}",
                frame, state, dispcnt
            );
        }
    }

    gba.input.release_key(KeyState::START);
    gba.input.release_key(KeyState::A);

    // Continue running to see what happens
    println!("\n=== After release ===");
    for frame in 400..600 {
        gba.run_frame_parallel(&mut fb);
        let state = gba.mem.read_word(0x02000074);
        let dispcnt = u16::from_le_bytes([gba.mem.io()[0], gba.mem.io()[1]]);
        if frame % 20 == 0 || state != 1 {
            println!(
                "Frame {:4}: State={:08X} DISPCNT={:04X}",
                frame, state, dispcnt
            );
        }
    }

    // Save screenshot
    let mut img_buf = vec![0u8; 240 * 160 * 3];
    for y in 0..160 {
        for x in 0..240 {
            let pixel = fb[y * 240 + x];
            let idx = (y * 240 + x) * 3;
            img_buf[idx] = ((pixel >> 16) & 0xFF) as u8;
            img_buf[idx + 1] = ((pixel >> 8) & 0xFF) as u8;
            img_buf[idx + 2] = (pixel & 0xFF) as u8;
        }
    }

    // Save as PPM then convert
    let ppm = format!("P6\n240 160\n255\n");
    std::fs::write(
        "/tmp/start_result.ppm",
        ppm.as_bytes()
            .iter()
            .chain(img_buf.iter())
            .cloned()
            .collect::<Vec<u8>>(),
    )
    .unwrap();
}

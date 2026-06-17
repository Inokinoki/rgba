use rgba::Gba;
use rgba::KeyState;

fn save_bmp(framebuffer: &[u32], path: &str) {
    let width = 240u32;
    let height = 160u32;
    let row_size = (width * 4 + 3) & !3;
    let file_size = 54 + row_size * height;
    let mut bmp = vec![0u8; file_size as usize];
    bmp[0..2].copy_from_slice(b"BM");
    bmp[2..6].copy_from_slice(&file_size.to_le_bytes());
    bmp[10..14].copy_from_slice(&54u32.to_le_bytes());
    bmp[14..18].copy_from_slice(&40u32.to_le_bytes());
    bmp[18..22].copy_from_slice(&width.to_le_bytes());
    bmp[22..26].copy_from_slice(&height.to_le_bytes());
    bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
    bmp[28..30].copy_from_slice(&32u16.to_le_bytes());
    for y in 0..height {
        for x in 0..width {
            let src_idx = ((height - 1 - y) * width + x) as usize;
            let dst_idx = (54 + y * row_size + x * 4) as usize;
            let pixel = framebuffer[src_idx];
            bmp[dst_idx] = (pixel & 0xFF) as u8;
            bmp[dst_idx + 1] = ((pixel >> 8) & 0xFF) as u8;
            bmp[dst_idx + 2] = ((pixel >> 16) & 0xFF) as u8;
            bmp[dst_idx + 3] = ((pixel >> 24) & 0xFF) as u8;
        }
    }
    std::fs::write(path, &bmp).unwrap();
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];

    for _ in 0..240 {
        gba.run_frame_parallel(&mut framebuffer);
    }
    gba.input_mut().press_key(KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut framebuffer);
    }
    gba.input_mut().release_key(KeyState::START);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    // Aggressively mash A to get through ALL dialogue (1000 rounds)
    for i in 0..1000 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..3 {
            gba.run_frame_parallel(&mut framebuffer);
        }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..12 {
            gba.run_frame_parallel(&mut framebuffer);
        }

        if i % 100 == 0 {
            let unique: std::collections::HashMap<u32, u32> =
                framebuffer
                    .iter()
                    .fold(std::collections::HashMap::new(), |mut m, &p| {
                        *m.entry(p).or_insert(0) += 1;
                        m
                    });
            println!("A-round {}: {} colors", i, unique.len());
        }
    }

    // Now we should be in gameplay
    save_bmp(&framebuffer, "/tmp/after_dialogue.bmp");
    let unique: std::collections::HashMap<u32, u32> =
        framebuffer
            .iter()
            .fold(std::collections::HashMap::new(), |mut m, &p| {
                *m.entry(p).or_insert(0) += 1;
                m
            });
    println!("\nAfter dialogue: {} colors", unique.len());

    // Walk around
    for step in 0..100 {
        let dir = match step % 4 {
            0 => KeyState::DOWN,
            1 => KeyState::RIGHT,
            2 => KeyState::UP,
            _ => KeyState::LEFT,
        };
        gba.input_mut().press_key(dir);
        for _ in 0..5 {
            gba.run_frame_parallel(&mut framebuffer);
        }
        gba.input_mut().release_key(dir);
        for _ in 0..10 {
            gba.run_frame_parallel(&mut framebuffer);
        }

        if step % 20 == 0 {
            save_bmp(&framebuffer, &format!("/tmp/gameplay_{}.bmp", step));
            let unique: std::collections::HashMap<u32, u32> =
                framebuffer
                    .iter()
                    .fold(std::collections::HashMap::new(), |mut m, &p| {
                        *m.entry(p).or_insert(0) += 1;
                        m
                    });
            let non_black = framebuffer.iter().filter(|&&p| p != 0).count();
            println!(
                "Step {}: {} colors, {} non-black",
                step,
                unique.len(),
                non_black
            );
        }
    }
}

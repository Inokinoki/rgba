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

fn color_count(framebuffer: &[u32]) -> (usize, usize) {
    use std::collections::HashMap;
    let unique: HashMap<u32, u32> = framebuffer.iter().fold(HashMap::new(), |mut m, &p| {
        *m.entry(p).or_insert(0) += 1;
        m
    });
    let non_black = framebuffer.iter().filter(|&&p| p != 0).count();
    (unique.len(), non_black)
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];

    // Skip intro - boot + title
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

    // Phase 1: Tap A slowly, capture every 10 presses
    println!("=== Phase 1: Slow A presses (60f between) ===");
    for i in 0..100 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..5 {
            gba.run_frame_parallel(&mut framebuffer);
        }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..55 {
            gba.run_frame_parallel(&mut framebuffer);
        }

        if i % 10 == 0 {
            let (colors, non_black) = color_count(&framebuffer);
            save_bmp(&framebuffer, &format!("/tmp/flow_a{}.bmp", i));
            println!("A-{}: {} colors, {} non-black", i, colors, non_black);
        }
    }

    // Phase 2: If stuck at 1 color, try pressing other buttons
    println!("\n=== Phase 2: Try other buttons ===");
    let (colors, _) = color_count(&framebuffer);
    if colors <= 3 {
        // Maybe it needs a different button or the naming screen needs special input
        println!("Stuck at {} colors, trying B button...", colors);
        for _ in 0..10 {
            gba.input_mut().press_key(KeyState::B);
            for _ in 0..5 {
                gba.run_frame_parallel(&mut framebuffer);
            }
            gba.input_mut().release_key(KeyState::B);
            for _ in 0..55 {
                gba.run_frame_parallel(&mut framebuffer);
            }
        }
        let (colors, non_black) = color_count(&framebuffer);
        save_bmp(&framebuffer, "/tmp/flow_after_b.bmp");
        println!("After B: {} colors, {} non-black", colors, non_black);

        // Try START
        gba.input_mut().press_key(KeyState::START);
        for _ in 0..5 {
            gba.run_frame_parallel(&mut framebuffer);
        }
        gba.input_mut().release_key(KeyState::START);
        for _ in 0..55 {
            gba.run_frame_parallel(&mut framebuffer);
        }
        let (colors, non_black) = color_count(&framebuffer);
        save_bmp(&framebuffer, "/tmp/flow_after_start.bmp");
        println!("After START: {} colors, {} non-black", colors, non_black);

        // Try SELECT
        gba.input_mut().press_key(KeyState::SELECT);
        for _ in 0..5 {
            gba.run_frame_parallel(&mut framebuffer);
        }
        gba.input_mut().release_key(KeyState::SELECT);
        for _ in 0..55 {
            gba.run_frame_parallel(&mut framebuffer);
        }
        let (colors, non_black) = color_count(&framebuffer);
        save_bmp(&framebuffer, "/tmp/flow_after_select.bmp");
        println!("After SELECT: {} colors, {} non-black", colors, non_black);

        // Try D-pad
        for dir in [
            KeyState::UP,
            KeyState::DOWN,
            KeyState::LEFT,
            KeyState::RIGHT,
        ] {
            gba.input_mut().press_key(dir);
            for _ in 0..5 {
                gba.run_frame_parallel(&mut framebuffer);
            }
            gba.input_mut().release_key(dir);
            for _ in 0..30 {
                gba.run_frame_parallel(&mut framebuffer);
            }
        }
        let (colors, non_black) = color_count(&framebuffer);
        save_bmp(&framebuffer, "/tmp/flow_after_dpad.bmp");
        println!("After D-pad: {} colors, {} non-black", colors, non_black);
    }

    // Phase 3: Fresh start with just 30 A presses to see the early screens
    println!("\n=== Phase 3: Fresh start, 30 A presses, capture every 3 ===");
    let mut gba2 = Gba::new();
    gba2.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb2 = vec![0u32; 240 * 160];

    for _ in 0..240 {
        gba2.run_frame_parallel(&mut fb2);
    }
    gba2.input_mut().press_key(KeyState::START);
    for _ in 0..60 {
        gba2.run_frame_parallel(&mut fb2);
    }
    gba2.input_mut().release_key(KeyState::START);
    for _ in 0..120 {
        gba2.run_frame_parallel(&mut fb2);
    }

    for i in 0..30 {
        gba2.input_mut().press_key(KeyState::A);
        for _ in 0..5 {
            gba2.run_frame_parallel(&mut fb2);
        }
        gba2.input_mut().release_key(KeyState::A);
        for _ in 0..55 {
            gba2.run_frame_parallel(&mut fb2);
        }

        let (colors, non_black) = color_count(&fb2);
        save_bmp(&fb2, &format!("/tmp/early_a{}.bmp", i));
        println!("Early A-{}: {} colors, {} non-black", i, colors, non_black);
    }
}

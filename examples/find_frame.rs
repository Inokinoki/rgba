use rgba::Gba;
use rgba::KeyState;

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

    let mut best_frame = 0u32;
    let mut best_colors = 0usize;
    let mut best_fb = framebuffer.clone();

    for round in 0..60u32 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..3 {
            gba.run_frame_parallel(&mut framebuffer);
        }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..12 {
            gba.run_frame_parallel(&mut framebuffer);
        }

        gba.sync_ppu_full();
        let dispcnt = gba.ppu().get_dispcnt();

        if dispcnt & 0x80 != 0 {
            continue;
        }

        let enabled_bits = (dispcnt >> 8) & 0xF;
        if enabled_bits == 0 {
            continue;
        }

        let mut unique = std::collections::HashSet::new();
        for &p in &framebuffer {
            unique.insert(p);
        }

        let color_count = unique.len();
        let has_bg = (enabled_bits & 0xF) != 0;
        let has_obj = (dispcnt & 0x1000) != 0;
        let score = if has_bg && has_obj {
            color_count * 3
        } else if has_bg {
            color_count * 2
        } else {
            color_count
        };

        println!(
            "Round {}: dispcnt={:#06X} colors={} enabled_bg={:#X} obj={} score={}",
            round, dispcnt, color_count, enabled_bits, has_obj, score
        );

        if score > best_colors {
            best_colors = score;
            best_frame = round;
            best_fb = framebuffer.clone();
        }

        if round >= 15 && best_colors > 0 {
            break;
        }
    }

    println!(
        "\nBest frame: round {} with {} score",
        best_frame, best_colors
    );

    let mut unique_colors = std::collections::HashMap::new();
    for &p in &best_fb {
        *unique_colors.entry(p).or_insert(0u32) += 1;
    }
    let mut colors: Vec<_> = unique_colors.iter().collect();
    colors.sort_by(|a, b| b.1.cmp(a.1));
    for (i, (color, count)) in colors.iter().take(8).enumerate() {
        let r = ((**color >> 16) & 0xFF) as u16 * 31 / 255;
        let g = ((**color >> 8) & 0xFF) as u16 * 31 / 255;
        let b = (**color & 0xFF) as u16 * 31 / 255;
        println!(
            "  #{}: {:#010X} rgb555={:#06X} count={} ({:.1}%)",
            i + 1,
            color,
            r | (g << 5) | (b << 10),
            count,
            **count as f64 / 38400.0 * 100.0
        );
    }

    let row_size = ((240u32 * 4 + 3) & !3) as usize;
    let file_size = 54 + row_size * 160;
    let mut bmp = vec![0u8; file_size];
    bmp[0..2].copy_from_slice(b"BM");
    bmp[2..6].copy_from_slice(&(file_size as u32).to_le_bytes());
    bmp[10..14].copy_from_slice(&54u32.to_le_bytes());
    bmp[14..18].copy_from_slice(&40u32.to_le_bytes());
    bmp[18..22].copy_from_slice(&240u32.to_le_bytes());
    bmp[22..26].copy_from_slice(&160u32.to_le_bytes());
    bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
    bmp[28..30].copy_from_slice(&32u16.to_le_bytes());
    for y in 0..160u32 {
        for x in 0..240u32 {
            let src_idx = ((159 - y) * 240 + x) as usize;
            let dst_idx = (54 + y * row_size as u32 + x * 4) as usize;
            let pixel = best_fb[src_idx];
            bmp[dst_idx] = (pixel & 0xFF) as u8;
            bmp[dst_idx + 1] = ((pixel >> 8) & 0xFF) as u8;
            bmp[dst_idx + 2] = ((pixel >> 16) & 0xFF) as u8;
        }
    }
    std::fs::write("/tmp/bg_best.bmp", &bmp).unwrap();
    println!("\nSaved best frame to /tmp/bg_best.bmp");
}

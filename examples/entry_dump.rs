use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().press_key(KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(KeyState::START);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut fb);
    }
    for _ in 0..40 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..5 {
            gba.run_frame_parallel(&mut fb);
        }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..55 {
            gba.run_frame_parallel(&mut fb);
        }
    }

    let vram = gba.ppu().vram();

    // Check each BG layer's screen entries
    for bg in 1..=3 {
        let bgcnt = gba.ppu().get_bgcnt(bg);
        let tile_base = ((bgcnt >> 2) & 0x3) as usize * 0x4000;
        let map_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
        let priority = bgcnt & 0x3;
        let is_8bpp = (bgcnt >> 7) & 1;

        println!(
            "\n=== BG{}: cnt=0x{:04X} pri={} tile=0x{:04X} map=0x{:04X} {}bpp ===",
            bg,
            bgcnt,
            priority,
            tile_base,
            map_base,
            if is_8bpp != 0 { 8 } else { 4 }
        );

        // Show first 4 rows of screen entries (30 entries per row)
        for ty in 0..4 {
            print!("  row {}: ", ty);
            for tx in 0..30 {
                let offset = map_base + (ty * 32 + tx) * 2;
                if offset + 1 < vram.len() {
                    let entry = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
                    let tile_num = entry & 0x3FF;
                    let pal = (entry >> 12) & 0xF;
                    if tile_num != 0 {
                        print!("{:03}.{} ", tile_num, pal);
                    } else {
                        print!("--- ");
                    }
                }
            }
            println!();
        }

        // Count unique entries
        let mut entries = std::collections::HashMap::new();
        for i in 0..1024 {
            let offset = map_base + i * 2;
            if offset + 1 < vram.len() {
                let entry = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
                *entries.entry(entry).or_insert(0) += 1;
            }
        }
        let mut sorted: Vec<_> = entries.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        println!("  Top entries:");
        for (entry, count) in sorted.iter().take(5) {
            let tile_num = *entry & 0x3FF;
            let pal = (*entry >> 12) & 0xF;
            println!(
                "    T{}.P{} = {} occurrences ({:.1}%)",
                tile_num,
                pal,
                count,
                **count as f64 / 1024.0 * 100.0
            );
        }

        // Show pixel data of the most common non-zero tile
        if let Some((entry, _)) = sorted.iter().find(|(e, _)| **e != 0) {
            let tile_num = (*entry & 0x3FF) as usize;
            let pal = (*entry >> 12) & 0xF;
            let tile_off = tile_base + tile_num * 32;
            println!("  Tile {} (pal {}) data:", tile_num, pal);
            for row in 0..8 {
                let b0 = vram[tile_off + row * 4];
                let b1 = vram[tile_off + row * 4 + 1];
                let b2 = vram[tile_off + row * 4 + 2];
                let b3 = vram[tile_off + row * 4 + 3];
                print!("    ");
                for b in [b0, b1, b2, b3] {
                    print!("{:X}{:X}", b & 0xF, (b >> 4) & 0xF);
                }
                println!();
            }
        }
    }

    // Check if the game is rendering at A-40 by looking at the framebuffer
    let unique: std::collections::HashMap<u32, u32> =
        fb.iter()
            .fold(std::collections::HashMap::new(), |mut m, &p| {
                *m.entry(p).or_insert(0) += 1;
                m
            });
    let mut sorted_fb: Vec<_> = unique.iter().collect();
    sorted_fb.sort_by(|a, b| b.1.cmp(a.1));
    println!("\n=== Top framebuffer colors ===");
    for (color, count) in sorted_fb.iter().take(10) {
        let r = (**color >> 16) & 0xFF;
        let g = (**color >> 8) & 0xFF;
        let b = **color & 0xFF;
        println!(
            "  RGB({},{},{}) = {} pixels ({:.1}%)",
            r,
            g,
            b,
            count,
            **count as f64 / 38400.0 * 100.0
        );
    }
}

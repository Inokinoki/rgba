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

    let mut prev_dispcnt = 0u16;
    for round in 0..200 {
        if round % 10 < 7 {
            gba.input_mut().press_key(KeyState::A);
            for _ in 0..3 {
                gba.run_frame_parallel(&mut framebuffer);
            }
            gba.input_mut().release_key(KeyState::A);
        } else {
            let dir = match round % 4 {
                0 => KeyState::UP,
                1 => KeyState::DOWN,
                2 => KeyState::LEFT,
                _ => KeyState::RIGHT,
            };
            gba.input_mut().press_key(dir);
            for _ in 0..3 {
                gba.run_frame_parallel(&mut framebuffer);
            }
            gba.input_mut().release_key(dir);
        }
        for _ in 0..12 {
            gba.run_frame_parallel(&mut framebuffer);
        }

        if round >= 73 && round <= 92 {
            let dispcnt = gba.mem_mut().read_half(0x0400_0000);

            // Log state changes
            if dispcnt != prev_dispcnt {
                println!(
                    "Round {}: DISPCNT changed {:#X} -> {:#X}",
                    round, prev_dispcnt, dispcnt
                );
                prev_dispcnt = dispcnt;
            }

            if round == 75 || round == 78 {
                let ppu = gba.ppu();
                let vram = ppu.vram();
                println!("\n=== Round {} ===", round);
                println!("DISPCNT={:#X}", dispcnt);
                for bg in 0..4 {
                    let bgcnt = ppu.get_bgcnt(bg);
                    let tile_base = ppu.get_bg_tile_base(bg);
                    let map_base = ppu.get_bg_map_base(bg);
                    let hofs = ppu.get_bg_hofs(bg);
                    let vofs = ppu.get_bg_vofs(bg);
                    let priority = bgcnt & 3;
                    let size = (bgcnt >> 14) & 3;
                    let is_8bpp = (bgcnt & 0x80) != 0;
                    let enabled = (dispcnt >> (8 + bg)) & 1;
                    println!(
                        "  BG{}: en={} pri={} tb={:#X} mb={:#X} ho={} vo={} sz={} {}",
                        bg,
                        enabled,
                        priority,
                        tile_base,
                        map_base,
                        hofs,
                        vofs,
                        size,
                        if is_8bpp { "8bpp" } else { "4bpp" }
                    );
                }

                // Check VRAM non-zero stats
                let regions = [
                    ("Tile 0", 0x0000usize, 0x4000),
                    ("Tile 1", 0x4000, 0x8000),
                    ("Tile 2", 0x8000, 0xC000),
                    ("Tile 3", 0xC000, 0x10000),
                    ("SB 0x1C", 0xE000, 0xE800),
                    ("SB 0x1D", 0xE800, 0xF000),
                    ("SB 0x1E", 0xF000, 0xF800),
                    ("SB 0x1F", 0xF800, 0x10000),
                ];
                for (name, start, end) in &regions {
                    let nz: u32 = (*start..*end)
                        .map(|i| if vram[i] != 0 { 1u32 } else { 0 })
                        .sum();
                    if nz > 0 {
                        println!("  VRAM {}: {}/{}", name, nz, end - start);
                    }
                }

                // Check screen entries for BG3
                let mb3 = ppu.get_bg_map_base(3) as usize;
                let mut entries = Vec::new();
                for i in 0..32 {
                    let e = u16::from_le_bytes([vram[mb3 + i * 2], vram[mb3 + i * 2 + 1]]);
                    entries.push(e);
                }
                println!("  BG3 entries [0..32]: {:?}", &entries[..16]);

                let mb2 = ppu.get_bg_map_base(2) as usize;
                let mut entries2 = Vec::new();
                for i in 0..32 {
                    let e = u16::from_le_bytes([vram[mb2 + i * 2], vram[mb2 + i * 2 + 1]]);
                    entries2.push(e);
                }
                println!("  BG2 entries [0..32]: {:?}", &entries2[..16]);

                // Count unique colors
                let unique_colors: std::collections::HashMap<u32, u32> =
                    framebuffer
                        .iter()
                        .fold(std::collections::HashMap::new(), |mut m, &p| {
                            *m.entry(p).or_insert(0) += 1;
                            m
                        });
                println!("  Framebuffer: {} unique colors", unique_colors.len());
                save_bmp(&framebuffer, &format!("/tmp/compare_r{}.bmp", round));
            }
        }
    }
}

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
    gba.input.press_key(KeyState::START);
    for _ in 0..4 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input.release_key(KeyState::START);
    for _ in 0..180 {
        gba.run_frame_parallel(&mut fb);
    }

    // Now at frame 424 (title screen with some corruption)
    println!("=== FRAME 424 (title screen) ===");
    dump_state(&gba);

    // Press A 6 times to reach solid green frame
    for i in 0..6 {
        gba.input.press_key(KeyState::A);
        for _ in 0..4 {
            gba.run_frame_parallel(&mut fb);
        }
        gba.input.release_key(KeyState::A);
        for _ in 0..16 {
            gba.run_frame_parallel(&mut fb);
        }
    }

    println!("\n=== FRAME ~544 (after A x6, should be solid green) ===");
    dump_state(&gba);

    // Check what color the rendering produces for pixel (0,0)
    let ppu = &gba.ppu;
    let io = gba.mem.io();
    let dispcnt = u16::from_le_bytes([io[0], io[1]]);
    let mode = (dispcnt & 0x7) as u8;
    println!("\nMode: {}", mode);

    // Check backdrop color
    let pal = gba.mem.palette();
    let backdrop = u16::from_le_bytes([pal[0], pal[1]]);
    let r = (backdrop & 0x1F) as u32 * 255 / 31;
    let g = ((backdrop >> 5) & 0x1F) as u32 * 255 / 31;
    let b = ((backdrop >> 10) & 0x1F) as u32 * 255 / 31;
    println!(
        "Backdrop color: {:04X} -> RGB({}, {}, {})",
        backdrop, r, g, b
    );

    // Also check pixel (120, 80) - center of screen
    let center_color = gba.get_pixel_tile_mode(120, 80);
    let r = ((center_color & 0x1F) as u32 * 255 / 31) << 16;
    let g = (((center_color >> 5) & 0x1F) as u32 * 255 / 31) << 8;
    let b = ((center_color >> 10) & 0x1F) as u32 * 255 / 31;
    println!(
        "Pixel (120,80): color={:04X} fb={:08X}",
        center_color,
        r | g | b
    );
}

fn dump_state(gba: &Gba) {
    let io = gba.mem.io();
    let dispcnt = u16::from_le_bytes([io[0], io[1]]);
    println!("DISPCNT: {:04X} (mode={})", dispcnt, dispcnt & 7);

    for bg in 0..4 {
        let off = 8 + bg * 2;
        let cnt = u16::from_le_bytes([io[off], io[off + 1]]);
        let h_off = 16 + bg * 4;
        let v_off = h_off + 2;
        let hofs = u16::from_le_bytes([io[h_off], io[h_off + 1]]) & 0x1FF;
        let vofs = u16::from_le_bytes([io[v_off], io[v_off + 1]]) & 0x1FF;
        let char_base = ((cnt >> 2) & 3) * 0x4000;
        let screen_base = ((cnt >> 8) & 0x1F) * 0x800;
        let pri = cnt & 3;
        let is_256 = (cnt >> 7) & 1;
        let size = (cnt >> 14) & 3;
        let enabled = (dispcnt >> (8 + bg)) & 1;
        println!(
            "  BG{}: {} pri={} char={:05X} screen={:05X} {}bit size={} hofs={} vofs={}",
            bg,
            if enabled != 0 { "ON " } else { "OFF" },
            pri,
            0x06000000u32 + char_base as u32,
            0x06000000u32 + screen_base as u32,
            if is_256 != 0 { "256" } else { "16" },
            size,
            hofs,
            vofs
        );
    }

    // Check if each BG has screen entries
    let vram = gba.mem.vram();
    for bg in 0..4 {
        let off = 8 + bg * 2;
        let cnt = u16::from_le_bytes([io[off], io[off + 1]]);
        let enabled = (dispcnt >> (8 + bg)) & 1;
        if enabled == 0 {
            continue;
        }

        let screen_base = ((cnt >> 8) & 0x1F) as usize * 0x800;
        let mut non_zero = 0;
        for i in 0..1024 {
            let entry =
                u16::from_le_bytes([vram[screen_base + i * 2], vram[screen_base + i * 2 + 1]]);
            if entry != 0 {
                non_zero += 1;
            }
        }
        println!("  BG{}: {} non-zero screen entries", bg, non_zero);

        // Print first few non-zero entries
        let mut count = 0;
        for i in 0..1024 {
            if count >= 10 {
                break;
            }
            let entry =
                u16::from_le_bytes([vram[screen_base + i * 2], vram[screen_base + i * 2 + 1]]);
            if entry != 0 {
                let tile = entry & 0x3FF;
                let pal = (entry >> 12) & 0xF;
                println!("    SE[{}] = {:04X} (tile={} pal={})", i, entry, tile, pal);
                count += 1;
            }
        }
    }

    // Palette
    let pal = gba.mem.palette();
    let mut pal_nonzero = 0;
    for i in 0..256 {
        let c = u16::from_le_bytes([pal[i * 2], pal[i * 2 + 1]]);
        if c != 0 {
            pal_nonzero += 1;
        }
    }
    println!("  Palette: {} non-zero entries (of 256)", pal_nonzero);

    // Print first 32 non-zero palette entries
    let mut count = 0;
    for i in 0..256 {
        if count >= 32 {
            break;
        }
        let c = u16::from_le_bytes([pal[i * 2], pal[i * 2 + 1]]);
        if c != 0 {
            let r = c & 0x1F;
            let g = (c >> 5) & 0x1F;
            let b = (c >> 10) & 0x1F;
            println!("    PAL[{}] = {:04X} (R{} G{} B{})", i, c, r, g, b);
            count += 1;
        }
    }

    // Check tile data at first referenced tile
    for bg in 0..4 {
        let off = 8 + bg * 2;
        let cnt = u16::from_le_bytes([io[off], io[off + 1]]);
        let enabled = (dispcnt >> (8 + bg)) & 1;
        if enabled == 0 {
            continue;
        }
        if bg > 0 {
            break;
        } // Only check BG0

        let screen_base = ((cnt >> 8) & 0x1F) as usize * 0x800;
        let first_entry = u16::from_le_bytes([vram[screen_base], vram[screen_base + 1]]);
        if first_entry == 0 {
            continue;
        }

        let tile = (first_entry & 0x3FF) as usize;
        let char_base = ((cnt >> 2) & 3) as usize * 0x4000;
        let tile_offset = char_base + tile * 32;

        println!(
            "  BG0 first tile {} at offset {:05X}:",
            tile,
            0x06000000u32 + tile_offset as u32
        );
        if tile_offset + 32 <= vram.len() {
            let mut has_data = false;
            for j in 0..32 {
                if vram[tile_offset + j] != 0 {
                    has_data = true;
                    break;
                }
            }
            if has_data {
                println!(
                    "    HAS DATA: {:02X} {:02X} {:02X} {:02X} ...",
                    vram[tile_offset],
                    vram[tile_offset + 1],
                    vram[tile_offset + 2],
                    vram[tile_offset + 3]
                );
            } else {
                println!("    ALL ZEROS (empty tile)");
            }
        }
    }
}

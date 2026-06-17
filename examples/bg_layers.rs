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
    for _ in 0..100 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..3 {
            gba.run_frame_parallel(&mut framebuffer);
        }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..12 {
            gba.run_frame_parallel(&mut framebuffer);
        }
    }

    let vram = gba.mem().vram();
    let palette = gba.mem().palette();

    let bg_configs: Vec<(u32, u32, u32, u32, u32)> = vec![
        (0, 0xC000, 0xF800, 0, 432),
        (1, 0x0000, 0xF000, 2, 6),
        (2, 0x0000, 0xE800, 2, 6),
        (3, 0x0000, 0xE000, 2, 6),
    ];

    for &(bg, tile_base, scr_base, hofs, vofs) in &bg_configs {
        println!(
            "\n=== BG{} tile_base={:#X} scr_base={:#X} hofs={} vofs={} ===",
            bg, tile_base, scr_base, hofs, vofs
        );

        print!("  Entries: ");
        for ty in 0..3u32 {
            for tx in 0..6u32 {
                let map_x = (tx * 8 + hofs) / 8;
                let map_y = (ty * 8 + vofs) / 8;
                let screen_block = (map_y / 32) * 32 + (map_x / 32);
                let entry_off =
                    (scr_base + screen_block * 0x800 + ((map_y % 32) * 32 + (map_x % 32)) * 2)
                        as usize;
                if entry_off + 1 < vram.len() {
                    let entry = u16::from_le_bytes([vram[entry_off], vram[entry_off + 1]]);
                    let tile_num = entry & 0x3FF;
                    let pal = (entry >> 12) & 0xF;
                    print!("({},{})T{}P{} ", tx, ty, tile_num, pal);
                }
            }
            println!();
            print!("           ");
        }
        println!();

        let mut seen = std::collections::HashSet::new();
        for ty in 0..20u32 {
            for tx in 0..30u32 {
                let map_x = (tx * 8 + hofs) / 8;
                let map_y = (ty * 8 + vofs) / 8;
                let screen_block = (map_y / 32) * 32 + (map_x / 32);
                let entry_off =
                    (scr_base + screen_block * 0x800 + ((map_y % 32) * 32 + (map_x % 32)) * 2)
                        as usize;
                if entry_off + 1 < vram.len() {
                    let entry = u16::from_le_bytes([vram[entry_off], vram[entry_off + 1]]);
                    let tile_num = (entry & 0x3FF) as u32;
                    let pal = ((entry >> 12) & 0xF) as usize;
                    if seen.insert(tile_num) && seen.len() <= 3 {
                        let toff = (tile_base + tile_num * 32) as usize;
                        if toff + 32 <= vram.len() {
                            let mut all_zero = true;
                            for i in 0..32 {
                                if vram[toff + i] != 0 {
                                    all_zero = false;
                                    break;
                                }
                            }
                            if all_zero {
                                println!("  Tile {} pal {}: ALL ZEROS", tile_num, pal);
                            } else {
                                print!("  Tile {} pal {}: ", tile_num, pal);
                                for i in 0..16 {
                                    print!("{:02X}", vram[toff + i]);
                                }
                                println!();
                            }
                        }
                    }
                }
            }
        }
        println!("  Unique tiles: {}", seen.len());
    }

    let backdrop = u16::from_le_bytes([palette[0], palette[1]]);
    let r = backdrop & 0x1F;
    let g = (backdrop >> 5) & 0x1F;
    let b = (backdrop >> 10) & 0x1F;
    println!("\n=== Backdrop (palette 0, color 0) ===");
    println!(
        "  {:#06X} R={} G={} B={} => #{:02X}{:02X}{:02X}",
        backdrop,
        r,
        g,
        b,
        r * 255 / 31,
        g * 255 / 31,
        b * 255 / 31
    );
}

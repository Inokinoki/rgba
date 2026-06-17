use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Run to frame 424 (same as seq_start)
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

    let io = gba.mem.io();
    let dispcnt = u16::from_le_bytes([io[0], io[1]]);
    let bg0cnt = u16::from_le_bytes([io[8], io[9]]);
    let bg1cnt = u16::from_le_bytes([io[0xA], io[0xB]]);
    let bg2cnt = u16::from_le_bytes([io[0xC], io[0xD]]);
    let bg3cnt = u16::from_le_bytes([io[0xE], io[0xF]]);

    println!("DISPCNT: {:04X}", dispcnt);
    println!("BG0CNT: {:04X}", bg0cnt);
    println!("BG1CNT: {:04X}", bg1cnt);
    println!("BG2CNT: {:04X}", bg2cnt);
    println!("BG3CNT: {:04X}", bg3cnt);

    for i in 0..4 {
        let cnt = [bg0cnt, bg1cnt, bg2cnt, bg3cnt][i];
        let priority = cnt & 3;
        let char_base = ((cnt >> 2) & 3) * 0x4000;
        let palette_mode = if (cnt >> 7) & 1 == 1 {
            "256-color"
        } else {
            "16-color"
        };
        let screen_base = ((cnt >> 8) & 0x1F) * 0x800;
        let size = (cnt >> 14) & 3;
        println!(
            "  BG{}: pri={} char_base={:05X} screen_base={:05X} {} size={}",
            i,
            priority,
            0x06000000u32 + char_base as u32,
            0x06000000u32 + screen_base as u32,
            palette_mode,
            size
        );
    }

    let pal = gba.mem.palette();
    println!("\n=== BG PALETTE (first 256 entries, non-zero) ===");
    for i in 0..256 {
        let c = u16::from_le_bytes([pal[i * 2], pal[i * 2 + 1]]);
        if c != 0 {
            let r = c & 0x1F;
            let g = (c >> 5) & 0x1F;
            let b = (c >> 10) & 0x1F;
            if i < 16 || (i >= 240 && i < 256) {
                println!("  PAL[{}] = {:04X} (R{} G{} B{})", i, c, r, g, b);
            }
        }
    }

    let vram = gba.mem.vram();

    for bg_idx in 0..4 {
        let cnt = [bg0cnt, bg1cnt, bg2cnt, bg3cnt][bg_idx];
        let screen_base = ((cnt >> 8) & 0x1F) as usize * 0x800;
        let char_base = ((cnt >> 2) & 3) as usize * 0x4000;

        let mut non_zero = 0;
        let mut pal_counts = [0u32; 16];
        for i in 0..1024 {
            let entry =
                u16::from_le_bytes([vram[screen_base + i * 2], vram[screen_base + i * 2 + 1]]);
            if entry != 0 {
                non_zero += 1;
                pal_counts[((entry >> 12) & 0xF) as usize] += 1;
            }
        }
        println!(
            "\nBG{} screen @ {:05X}: {} non-zero entries (of 1024)",
            bg_idx,
            0x06000000 + screen_base,
            non_zero
        );
        for p in 0..16 {
            if pal_counts[p] > 0 {
                println!("  palette {}: {} entries", p, pal_counts[p]);
            }
        }

        // Print first 40 non-zero screen entries
        let mut count = 0;
        for i in 0..1024 {
            let entry =
                u16::from_le_bytes([vram[screen_base + i * 2], vram[screen_base + i * 2 + 1]]);
            if entry != 0 && count < 40 {
                let tile = entry & 0x3FF;
                let hflip = (entry >> 10) & 1;
                let vflip = (entry >> 11) & 1;
                let pal_idx = (entry >> 12) & 0xF;
                println!(
                    "  SE[{}] = {:04X} (tile={} h={} v={} pal={})",
                    i, entry, tile, hflip, vflip, pal_idx
                );
                count += 1;
            }
        }

        // Check tile data
        let mut tile_data_found = 0;
        for i in 0..512 {
            let offset = char_base + i * 32;
            if offset + 32 <= vram.len() {
                let mut has_data = false;
                for j in 0..32 {
                    if vram[offset + j] != 0 {
                        has_data = true;
                        break;
                    }
                }
                if has_data {
                    tile_data_found += 1;
                }
            }
        }
        println!(
            "  char_base {:05X}: {} tiles with data (checked 0-511)",
            0x06000000 + char_base,
            tile_data_found
        );
    }
}

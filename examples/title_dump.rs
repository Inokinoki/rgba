use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    println!("=== Scanning frames to find title screen ===");
    for frame in 0..600u32 {
        gba.run_frame_parallel(&mut framebuffer);

        if frame < 230 {
            continue;
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

        let mode = dispcnt & 0x7;
        let has_bg = enabled_bits != 0;
        let has_obj = (dispcnt & 0x1000) != 0;
        let has_win0 = (dispcnt & 0x2000) != 0;

        println!(
            "Frame {}: dispcnt={:#06X} mode={} bg_en={:#05b} obj={} win0={} colors={}",
            frame,
            dispcnt,
            mode,
            enabled_bits,
            has_obj,
            has_win0,
            unique.len()
        );

        if frame >= 239 && frame <= 241 {
            println!("\n=== Detailed Dump at Frame {} ===", frame);

            let ppu = gba.ppu();
            let mem = gba.mem();
            let vram = ppu.vram();
            let io = mem.io();

            println!("\nDISPCNT: {:#06X}", dispcnt);
            for bg in 0..4usize {
                let bgcnt = ppu.get_bgcnt(bg);
                let enabled = ppu.is_bg_enabled(bg);
                let priority = bgcnt & 0x3;
                let tile_base = ((bgcnt >> 2) & 0x3) as usize * 0x4000;
                let map_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
                let bg_size = (bgcnt >> 14) & 0x3;
                let is_8bpp = (bgcnt & 0x80) != 0;
                let hofs = ppu.get_bg_hofs(bg);
                let vofs = ppu.get_bg_vofs(bg);
                println!("BG{}: en={} cnt={:#06X} pri={} tile={:#06X} map={:#06X} sz={} 8bpp={} hofs={} vofs={}",
                    bg, enabled, bgcnt, priority, tile_base, map_base, bg_size, is_8bpp, hofs, vofs);
            }

            let win0h = u16::from_le_bytes([io[0x40], io[0x41]]);
            let win0v = u16::from_le_bytes([io[0x42], io[0x43]]);
            let winin = u16::from_le_bytes([io[0x48], io[0x49]]);
            let winout = u16::from_le_bytes([io[0x4A], io[0x4B]]);
            let bldcnt = ppu.get_blend_control();
            println!("WIN0: h={:#06X} v={:#06X}", win0h, win0v);
            println!("WININ: {:#06X}  WINOUT: {:#06X}", winin, winout);
            println!("BLDCNT: {:#06X} mode={}", bldcnt, (bldcnt >> 6) & 3);

            for bg in 0..4usize {
                if !ppu.is_bg_enabled(bg) {
                    continue;
                }
                let bgcnt = ppu.get_bgcnt(bg);
                let tile_base = ((bgcnt >> 2) & 0x3) as usize * 0x4000;
                let map_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
                let hofs = ppu.get_bg_hofs(bg);
                let vofs = ppu.get_bg_vofs(bg);

                println!("\n--- BG{} Screen Entries (rows 0-4, cols 0-15) map_base={:#X} hofs={} vofs={} ---",
                    bg, map_base, hofs, vofs);

                for ty in 0..5u16 {
                    print!("  row {:2}: ", ty);
                    for tx in 0..16u16 {
                        let bg_x = tx * 8 + hofs;
                        let bg_y = ty * 8 + vofs;
                        let tile_x = bg_x / 8;
                        let tile_y = bg_y / 8;
                        let entry_off = map_base + (tile_y as usize * 32 + tile_x as usize) * 2;
                        let entry = if entry_off + 1 < vram.len() {
                            u16::from_le_bytes([vram[entry_off], vram[entry_off + 1]])
                        } else {
                            0xFFFF
                        };
                        let tile_num = entry & 0x3FF;
                        let pal = (entry >> 12) & 0xF;
                        if tile_num == 1023 {
                            print!(" .   ");
                        } else {
                            print!("{:4}p{} ", tile_num, pal);
                        }
                    }
                    println!();
                }

                let mut visible = 0u32;
                let mut tile_counts = std::collections::HashMap::new();
                for ty in 0..20u16 {
                    for tx in 0..30u16 {
                        let bg_x = tx * 8 + hofs;
                        let bg_y = ty * 8 + vofs;
                        let tile_x = bg_x / 8;
                        let tile_y = bg_y / 8;
                        let entry_off = map_base + (tile_y as usize * 32 + tile_x as usize) * 2;
                        let entry = if entry_off + 1 < vram.len() {
                            u16::from_le_bytes([vram[entry_off], vram[entry_off + 1]])
                        } else {
                            0
                        };
                        let tile_num = entry & 0x3FF;
                        *tile_counts.entry(tile_num).or_insert(0u32) += 1;
                        if tile_num != 0 && tile_num != 1023 {
                            let tile_off = tile_base + tile_num as usize * 32;
                            let has_data = tile_off + 4 <= vram.len()
                                && vram[tile_off..tile_off + 4].iter().any(|&b| b != 0);
                            if has_data {
                                visible += 1;
                            }
                        }
                    }
                }
                let mut tiles: Vec<_> = tile_counts.iter().collect();
                tiles.sort_by(|a, b| b.1.cmp(a.1));
                println!("  Visible non-empty tiles: {}/600", visible);
                print!("  Top tiles: ");
                for (tile, count) in tiles.iter().take(8) {
                    print!("{}({}) ", tile, count);
                }
                println!();
            }

            let backdrop = mem.read_palette_color(0, 0);
            let r = (backdrop & 0x1F) * 255 / 31;
            let g = ((backdrop >> 5) & 0x1F) * 255 / 31;
            let b = ((backdrop >> 10) & 0x1F) * 255 / 31;
            println!("\nBackdrop: {:#06X} RGB=({},{},{})", backdrop, r, g, b);

            let mut unique_colors = std::collections::HashMap::new();
            for &p in &framebuffer {
                *unique_colors.entry(p).or_insert(0u32) += 1;
            }
            let mut colors: Vec<_> = unique_colors.iter().collect();
            colors.sort_by(|a, b| b.1.cmp(a.1));
            println!("Top colors:");
            for (i, (color, count)) in colors.iter().take(10).enumerate() {
                let r5 = ((**color >> 16) & 0xFF) as u16 * 31 / 255;
                let g5 = ((**color >> 8) & 0xFF) as u16 * 31 / 255;
                let b5 = (**color & 0xFF) as u16 * 31 / 255;
                println!(
                    "  #{}: {:#010X} -> rgb555={:#06X} count={} ({:.1}%)",
                    i + 1,
                    color,
                    r5 | (g5 << 5) | (b5 << 10),
                    count,
                    **count as f64 / 38400.0 * 100.0
                );
            }

            let row_size = ((240u32 * 4 + 3) & !3) as usize;
            let file_size = 54 + row_size * 160;
            let mut bmp_data = vec![0u8; file_size];
            bmp_data[0..2].copy_from_slice(b"BM");
            bmp_data[2..6].copy_from_slice(&(file_size as u32).to_le_bytes());
            bmp_data[10..14].copy_from_slice(&54u32.to_le_bytes());
            bmp_data[14..18].copy_from_slice(&40u32.to_le_bytes());
            bmp_data[18..22].copy_from_slice(&240u32.to_le_bytes());
            bmp_data[22..26].copy_from_slice(&160u32.to_le_bytes());
            bmp_data[26..28].copy_from_slice(&1u16.to_le_bytes());
            bmp_data[28..30].copy_from_slice(&32u16.to_le_bytes());
            for y in 0..160u32 {
                for x in 0..240u32 {
                    let src_idx = ((159 - y) * 240 + x) as usize;
                    let dst_idx = (54 + y * row_size as u32 + x * 4) as usize;
                    let pixel = framebuffer[src_idx];
                    bmp_data[dst_idx] = (pixel & 0xFF) as u8;
                    bmp_data[dst_idx + 1] = ((pixel >> 8) & 0xFF) as u8;
                    bmp_data[dst_idx + 2] = ((pixel >> 16) & 0xFF) as u8;
                }
            }
            std::fs::write("/tmp/title_ours.bmp", &bmp_data).unwrap();
            println!("\nSaved title screen to /tmp/title_ours.bmp");
            break;
        }
    }
}

use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..3000 {
        gba.run_frame_parallel(&mut fb);
    }

    press(&mut gba, rgba::KeyState::START, 300, &mut fb);
    press(&mut gba, rgba::KeyState::A, 300, &mut fb);

    for _ in 0..4 {
        for _ in 0..40 {
            press(&mut gba, rgba::KeyState::RIGHT, 15, &mut fb);
        }
        press(&mut gba, rgba::KeyState::A, 200, &mut fb);
    }

    for _ in 0..100 {
        press(&mut gba, rgba::KeyState::A, 30, &mut fb);
    }

    gba.sync_ppu_full();

    let vram = gba.ppu().vram();
    let dc = gba.ppu().get_dispcnt();
    println!("DISPCNT: {:#06X}", dc);

    let mut tile_count = 0;
    let mut last_tile = 0u16;
    for t in 0..1024u16 {
        let start = t as usize * 32;
        if start + 32 > vram.len() {
            break;
        }
        let mut has = false;
        for b in 0..32 {
            if vram[start + b] != 0 {
                has = true;
                break;
            }
        }
        if has {
            tile_count += 1;
            last_tile = t;
        }
    }
    println!("Tiles with data: {} (last={})", tile_count, last_tile);

    let bg0cnt = gba.ppu().get_bgcnt(0);
    let bg1cnt = gba.ppu().get_bgcnt(1);
    let bg2cnt = gba.ppu().get_bgcnt(2);
    let bg3cnt = gba.ppu().get_bgcnt(3);
    for bg in 0..4 {
        let bgcnt = [bg0cnt, bg1cnt, bg2cnt, bg3cnt][bg];
        let enabled = (dc >> (8 + bg)) & 1;
        let pri = bgcnt & 3;
        let tile_base = ((bgcnt >> 2) & 3) * 0x4000;
        let scr_base = ((bgcnt >> 8) & 0x1F) * 0x800;
        let size = (bgcnt >> 14) & 3;
        println!(
            "BG{}: en={} pri={} tile_base={:#06X} scr_base={:#06X} size={}",
            bg, enabled, pri, tile_base, scr_base, size
        );
    }

    println!("\n--- BG2 screen entries (first 4 rows) ---");
    let bg2cnt = gba.ppu().get_bgcnt(2);
    let bg2_scr_base = ((bg2cnt >> 8) & 0x1F) * 0x800;
    let bg2_tile_base = ((bg2cnt >> 2) & 3) * 0x4000;
    let bg2_hofs = gba.ppu().get_bg_hofs(2);
    let bg2_vofs = gba.ppu().get_bg_vofs(2);
    println!("BG2 hofs={} vofs={}", bg2_hofs, bg2_vofs);

    for row in 0..6 {
        print!("Row {}: ", row);
        for col in 0..30 {
            let screen_x = (col + (bg2_hofs as usize / 8)) % 64;
            let screen_y = (row + (bg2_vofs as usize / 8)) % 64;
            let block_x = screen_x / 32;
            let block_y = screen_y / 32;
            let local_x = screen_x % 32;
            let local_y = screen_y % 32;
            let block = block_y * 2 + block_x;
            let offset = bg2_scr_base as usize + block * 0x800 + (local_y * 32 + local_x) * 2;
            let entry = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
            let tile_num = entry & 0x3FF;
            let pal = (entry >> 12) & 0xF;
            let has_data = {
                let tstart = bg2_tile_base as usize + tile_num as usize * 32;
                if tstart + 32 <= vram.len() {
                    vram[tstart..tstart + 32].iter().any(|&b| b != 0)
                } else {
                    false
                }
            };
            print!("{}{} ", tile_num, if has_data { "" } else { "!" });
        }
        println!();
    }

    println!("\n--- BG0 first visible tiles pixel data ---");
    let bg0cnt = gba.ppu().get_bgcnt(0);
    let bg0_scr_base = ((bg0cnt >> 8) & 0x1F) * 0x800;
    let bg0_tile_base = ((bg0cnt >> 2) & 3) * 0x4000;
    let bg0_hofs = gba.ppu().get_bg_hofs(0);
    let bg0_vofs = gba.ppu().get_bg_vofs(0);
    println!("BG0 hofs={} vofs={}", bg0_hofs, bg0_vofs);

    for row in 0..3 {
        for col in 0..8 {
            let screen_x = (col + (bg0_hofs as usize / 8)) % 64;
            let screen_y = (row + (bg0_vofs as usize / 8)) % 64;
            let block_x = screen_x / 32;
            let block_y = screen_y / 32;
            let local_x = screen_x % 32;
            let local_y = screen_y % 32;
            let block = block_y * 2 + block_x;
            let offset = bg0_scr_base as usize + block * 0x800 + (local_y * 32 + local_x) * 2;
            let entry = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
            let tile_num = entry & 0x3FF;
            let pal = (entry >> 12) & 0xF;

            let tstart = bg0_tile_base as usize + tile_num as usize * 32;
            let mut nonzero = 0;
            if tstart + 32 <= vram.len() {
                for b in 0..32 {
                    if vram[tstart + b] != 0 {
                        nonzero += 1;
                    }
                }
            }
            if nonzero > 0 || tile_num != 0 {
                println!(
                    "  ({},{}) entry={:#06X} tile={} pal={} data_bytes={}",
                    col, row, entry, tile_num, pal, nonzero
                );
            }
        }
    }

    println!("\n--- Palette BG colors (first 16) ---");
    let pal = gba.mem().palette();
    for i in 0..16 {
        let color = u16::from_le_bytes([pal[i * 2], pal[i * 2 + 1]]);
        let r = (color & 0x1F) as u32 * 255 / 31;
        let g = ((color >> 5) & 0x1F) as u32 * 255 / 31;
        let b = ((color >> 10) & 0x1F) as u32 * 255 / 31;
        print!("{}: RGB({},{},{}) ", i, r, g, b);
        if i % 4 == 3 {
            println!();
        }
    }

    println!("\n--- OBJ tile base check ---");
    let obj_tile_base = 0x10000;
    let mut obj_tiles = 0;
    for t in 0..512 {
        let start = obj_tile_base + t * 32;
        if start + 32 > vram.len() {
            break;
        }
        let mut has = false;
        for b in 0..32 {
            if vram[start + b] != 0 {
                has = true;
                break;
            }
        }
        if has {
            obj_tiles += 1;
        }
    }
    println!("OBJ VRAM tiles with data: {}", obj_tiles);
}

fn press(gba: &mut Gba, key: rgba::KeyState, frames: u32, fb: &mut [u32]) {
    gba.input_mut().press_key(key);
    for _ in 0..10 {
        gba.run_frame_parallel(fb);
    }
    gba.input_mut().release_key(key);
    for _ in 0..frames.saturating_sub(10) {
        gba.run_frame_parallel(fb);
    }
}

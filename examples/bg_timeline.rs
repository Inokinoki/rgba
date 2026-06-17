use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    println!("=== Frame-by-frame BG register analysis ===");
    let mut prev_dispcnt = 0u16;
    let mut prev_bgcnt = [0u16; 4];

    for frame in 0..350u32 {
        gba.run_frame_parallel(&mut framebuffer);

        let dispcnt = gba.ppu().get_dispcnt();
        let bgcnt: Vec<u16> = (0..4).map(|i| gba.ppu().get_bgcnt(i)).collect();

        if dispcnt != prev_dispcnt || bgcnt != prev_bgcnt {
            let mode = dispcnt & 0x7;
            let bg_en = (dispcnt >> 8) & 0xF;
            let obj_en = (dispcnt >> 12) & 1;
            println!(
                "\nFrame {}: DISPCNT={:#06X} mode={} BG_en={:#X} OBJ_en={}",
                frame, dispcnt, mode, bg_en, obj_en
            );

            for i in 0..4 {
                if bgcnt[i] != prev_bgcnt[i] {
                    let tile_base = ((bgcnt[i] as usize >> 2) & 0x3) * 0x4000;
                    let map_base = ((bgcnt[i] as usize >> 8) & 0x1F) * 0x800;
                    let size = (bgcnt[i] >> 14) & 3;
                    let prio = bgcnt[i] & 3;
                    let bpp = if (bgcnt[i] & 0x80) != 0 { 8 } else { 4 };
                    let bghofs = gba.ppu().get_bg_hofs(i);
                    let bgvofs = gba.ppu().get_bg_vofs(i);
                    println!("  BG{}: {:#06X} pri={} tile={:#06X} map={:#06X} size={} {}bpp hofs={} vofs={}",
                        i, bgcnt[i], prio, tile_base, map_base, size, bpp, bghofs, bgvofs);
                }
            }

            prev_dispcnt = dispcnt;
            prev_bgcnt.clone_from_slice(&bgcnt);
        }
    }

    println!("\n=== Final VRAM analysis ===");
    let vram = gba.mem().vram();

    for tile_base_addr in [0x0000usize, 0x4000, 0x8000, 0xC000] {
        let mut tiles_with_data = 0;
        let mut max_tile = 0u32;
        for tile_idx in 0..512u32 {
            let base = tile_base_addr + (tile_idx as usize) * 32;
            if base + 32 <= vram.len() {
                let has_data = (0..32).any(|j| vram[base + j] != 0);
                if has_data {
                    tiles_with_data += 1;
                    max_tile = tile_idx;
                }
            }
        }
        if tiles_with_data > 0 {
            println!(
                "Tile base {:#06X}: {}/512 tiles have data (max tile: {})",
                tile_base_addr, tiles_with_data, max_tile
            );
        }
    }

    for map_base_addr in [0xC000usize, 0xD000, 0xE000, 0xF000, 0xF800] {
        let mut nonzero_entries = 0;
        let mut tile_min = u16::MAX;
        let mut tile_max = 0u16;
        for i in (0..2048).step_by(2) {
            let off = map_base_addr + i;
            if off + 2 <= vram.len() {
                let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
                if entry != 0 {
                    nonzero_entries += 1;
                    let tile = entry & 0x3FF;
                    tile_min = tile_min.min(tile);
                    tile_max = tile_max.max(tile);
                }
            }
        }
        if nonzero_entries > 0 {
            println!(
                "Map base {:#06X}: {}/1024 nonzero entries, tiles {}..={}",
                map_base_addr, nonzero_entries, tile_min, tile_max
            );
        }
    }

    println!("\n=== Compare mGBA expected vs our VRAM ===");
    println!("mGBA title screen has: sky, grass, trees, characters, Chinese text");
    println!("Our emulator shows: solid green with scattered sprite fragments");
    println!("\nTile data at 0x0000 (first 256 bytes):");
    for row in 0..8 {
        let off = row * 32;
        print!("  {:#06X}: ", off);
        for b in 0..32 {
            if vram[off + b] != 0 {
                print!("{:02X}", vram[off + b]);
            } else {
                print!("..");
            }
        }
        println!();
    }

    println!("\nTile data at 0xC000 (first 256 bytes):");
    for row in 0..8 {
        let off = 0xC000 + row * 32;
        if off + 32 <= vram.len() {
            print!("  {:#06X}: ", off);
            for b in 0..32 {
                if vram[off + b] != 0 {
                    print!("{:02X}", vram[off + b]);
                } else {
                    print!("..");
                }
            }
            println!();
        }
    }
}

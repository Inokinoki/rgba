use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem_mut().vram_write_log.clear();

    for frame in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    gba.sync_ppu_full();
    let ppu = gba.ppu();
    let vram = ppu.vram();

    println!("=== Frame 200 title screen analysis ===");

    let map_base = 0xF000usize;
    println!("\nBG3 map (0xF000) first 4 rows:");
    for ty in 0..4 {
        for tx in 0..32 {
            let off = map_base + (ty * 32 + tx) * 2;
            let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
            let tile_num = entry & 0x3FF;
            print!("{:4} ", tile_num);
        }
        println!();
    }

    println!("\nBG3 map (0xF000) rows 18-22 (visible area with hofs=196):");
    for ty in 18..22 {
        for tx in 0..32 {
            let off = map_base + (ty * 32 + tx) * 2;
            let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
            let tile_num = entry & 0x3FF;
            print!("{:4} ", tile_num);
        }
        println!();
    }

    let mut tiles_used = std::collections::BTreeSet::new();
    for ty in 0..32 {
        for tx in 0..32 {
            let off = map_base + (ty * 32 + tx) * 2;
            let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
            let tile_num = entry & 0x3FF;
            tiles_used.insert(tile_num);
        }
    }
    println!("\nUnique tile numbers in BG3 map: {:?}", tiles_used);

    for &t in &tiles_used {
        let off = t as usize * 32;
        let mut all_aa = true;
        let mut all_zero = true;
        for b in 0..32 {
            if vram[off + b] != 0xAA {
                all_aa = false;
            }
            if vram[off + b] != 0 {
                all_zero = false;
            }
        }
        if !all_aa && !all_zero {
            print!("Tile {}: ", t);
            for b in 0..8 {
                print!("{:02X}", vram[off + b]);
            }
            println!("...");
        } else if all_aa {
            println!("Tile {}: ALL 0xAA", t);
        } else {
            println!("Tile {}: ALL 0x00", t);
        }
    }

    let map_c000 = 0xC000usize;
    println!("\nBG0 map (0xC000) first 4 rows:");
    for ty in 0..4 {
        for tx in 0..32 {
            let off = map_c000 + (ty * 32 + tx) * 2;
            let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
            let tile_num = entry & 0x3FF;
            print!("{:4} ", tile_num);
        }
        println!();
    }

    println!("\nBG0 map (0xC000) rows 18-22:");
    for ty in 18..22 {
        for tx in 0..32 {
            let off = map_c000 + (ty * 32 + tx) * 2;
            let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
            let tile_num = entry & 0x3FF;
            print!("{:4} ", tile_num);
        }
        println!();
    }
}

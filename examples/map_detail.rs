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
    for _ in 0..60 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input.release_key(KeyState::START);
    for _ in 0..180 {
        gba.run_frame_parallel(&mut fb);
    }
    for _ in 0..80 {
        gba.input.press_key(KeyState::A);
        for _ in 0..4 {
            gba.run_frame_parallel(&mut fb);
        }
        gba.input.release_key(KeyState::A);
        for _ in 0..16 {
            gba.run_frame_parallel(&mut fb);
        }
    }

    gba.sync_ppu_full();
    let ppu = gba.ppu();
    let vram = ppu.vram();

    println!("=== BG1 map (visible area with hofs=2, vofs=6) ===");
    let hofs: u16 = 2;
    let vofs: u16 = 6;
    for screen_y in 0..21u16 {
        for screen_x in 0..30u16 {
            let bg_x = ((screen_x as u32 * 8 + hofs as u32) % 256) as u16;
            let bg_y = ((screen_y as u32 * 8 + vofs as u32) % 256) as u16;
            let tile_x = bg_x / 8;
            let tile_y = bg_y / 8;
            let map_base = 0xF000usize;
            let entry_offset = map_base + (tile_y as usize * 32 + tile_x as usize) * 2;
            let entry = u16::from_le_bytes([vram[entry_offset], vram[entry_offset + 1]]);
            let tile_num = entry & 0x3FF;
            let palette = (entry >> 12) & 0xF;
            print!("{:4} ", tile_num);
        }
        println!();
    }

    println!("\n=== Full BG1 map dump (32x32) ===");
    for ty in 0..32 {
        for tx in 0..32 {
            let offset = 0xF000 + (ty * 32 + tx) * 2;
            let entry = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
            let tile_num = entry & 0x3FF;
            if tile_num != 1023 && tile_num != 0 {
                print!("({},{})={:#06X} ", tx, ty, entry);
            }
        }
    }
    println!();

    println!("\n=== BG1 map: unique nonzero entries ===");
    let mut unique = std::collections::BTreeSet::new();
    for ty in 0..32 {
        for tx in 0..32 {
            let offset = 0xF000 + (ty * 32 + tx) * 2;
            let entry = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
            unique.insert(entry);
        }
    }
    println!("{} unique entries", unique.len());
    for entry in &unique {
        let tile_num = entry & 0x3FF;
        let palette = (entry >> 12) & 0xF;
        println!(
            "  entry={:#06X} tile={} palette={}",
            entry, tile_num, palette
        );
    }

    println!(
        "\n=== Tile 1023 data at char_base 0 (offset {:#X}) ===",
        1023 * 32
    );
    let offset = 1023 * 32;
    for row in 0..8 {
        print!("  row {}: ", row);
        for b in 0..4 {
            print!("{:02X}", vram[offset + row * 4 + b]);
        }
        println!();
    }

    println!("\n=== BG3 full map (first 8 rows) ===");
    for ty in 0..8 {
        for tx in 0..32 {
            let offset = 0xE000 + (ty * 32 + tx) * 2;
            let entry = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
            let tile_num = entry & 0x3FF;
            print!("{:4} ", tile_num);
        }
        println!();
    }

    println!("\n=== Tile 279 data (BG3 tile) ===");
    let offset = 279 * 32;
    for row in 0..8 {
        print!("  row {}: ", row);
        for b in 0..4 {
            print!("{:02X}", vram[offset + row * 4 + b]);
        }
        println!();
    }
}

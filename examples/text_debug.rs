use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Get to the name entry screen
    for frame in 0..300u32 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..300 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut fb);
    }

    // Now check BG0 tile data and palette
    gba.sync_ppu_full();
    let ppu = gba.ppu();
    let vram = ppu.vram();
    let pal = gba.mem.palette();

    // BG0: tile_base=0, map_base=0xC000, priority=3
    // Check screen entries at map_base
    let map_base = 0xC000usize;
    println!("BG0 screen entries (first 32x32 = 1024 entries):");
    let mut non_zero_entries = 0;
    for i in 0..1024 {
        let entry = u16::from_le_bytes([vram[map_base + i * 2], vram[map_base + i * 2 + 1]]);
        if entry != 0 {
            non_zero_entries += 1;
            if non_zero_entries <= 20 {
                let tile_num = entry & 0x3FF;
                let pal_bank = (entry >> 12) & 0xF;
                let flip_h = (entry >> 10) & 1;
                let flip_v = (entry >> 11) & 1;
                let row = i / 32;
                let col = i % 32;
                println!(
                    "  [{:2},{:2}] tile={:4} pal={} fh={} fv={}",
                    row, col, tile_num, pal_bank, flip_h, flip_v
                );
            }
        }
    }
    println!("Non-zero BG0 entries: {}", non_zero_entries);

    // Check BG palette banks 0-15
    println!("\nBG Palette non-zero entries:");
    for bank in 0..16 {
        let base = bank * 16 * 2;
        let mut nz = 0;
        for i in 0..16 {
            let v = u16::from_le_bytes([pal[base + i * 2], pal[base + i * 2 + 1]]);
            if v != 0 {
                nz += 1;
            }
        }
        if nz > 0 {
            print!("  Bank {}: {}/16 colors:", bank, nz);
            for i in 0..4 {
                let v = u16::from_le_bytes([pal[base + i * 2], pal[base + i * 2 + 1]]);
                let r = v & 0x1F;
                let g = (v >> 5) & 0x1F;
                let b = (v >> 10) & 0x1F;
                print!(" [{:2}]={:04X}(r{}g{}b{})", i, v, r, g, b);
            }
            println!("...");
        }
    }

    // Check specific tiles mentioned in screen entries
    // If tile 0 has data, show it
    for tile in [0, 1] {
        let tile_off = tile * 32;
        let mut nz = 0;
        for b in 0..32 {
            if vram[tile_off + b] != 0 {
                nz += 1;
            }
        }
        if nz > 0 {
            print!("Tile {} data:", tile);
            for b in 0..32 {
                print!("{:02X}", vram[tile_off + b]);
            }
            println!(" ({} non-zero)", nz);
        }
    }
}

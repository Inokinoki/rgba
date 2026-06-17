use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();

    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let log = &gba.mem().vram_write_log;

    let mut tile_writes = vec![0u32; 512];
    for (addr, _pc, val) in log {
        let offset = (addr - 0x0600_0000) as usize;
        if offset < 0x4000 {
            let tile = offset / 32;
            tile_writes[tile] += 1;
        }
    }

    let mut tiles_with_writes = 0;
    for (i, &count) in tile_writes.iter().enumerate() {
        if count > 0 {
            tiles_with_writes += 1;
        }
    }
    println!("Tiles with writes in char block 0: {}", tiles_with_writes);

    println!("\nTiles with most writes:");
    let mut indexed: Vec<(usize, u32)> = tile_writes
        .iter()
        .enumerate()
        .map(|(i, &c)| (i, c))
        .collect();
    indexed.sort_by(|a, b| b.1.cmp(&a.1));
    for (tile, count) in indexed.iter().take(20) {
        let off = tile * 32;
        let vram = gba.mem().vram();
        let mut nonzero = 0;
        for b in 0..32 {
            if vram[off + b] != 0 {
                nonzero += 1;
            }
        }
        println!(
            "  Tile {}: {} writes, {} nonzero bytes",
            tile, count, nonzero
        );
    }

    let vram = gba.mem().vram();
    let bg0_tiles = [394, 403, 412, 420, 473, 482, 491, 499, 611, 629];
    println!("\nBG0 title screen tiles:");
    for &t in &bg0_tiles {
        if t < 512 {
            let off = t * 32;
            let mut nonzero = 0;
            for b in 0..32 {
                if vram[off + b] != 0 {
                    nonzero += 1;
                }
            }
            println!(
                "  Tile {}: {} writes, {} nonzero bytes",
                t, tile_writes[t], nonzero
            );
        } else {
            println!("  Tile {}: > 512, in char block 1", t);
        }
    }

    println!("\nFirst 20 writes to char block 0:");
    let mut count = 0;
    for (addr, pc, val) in log {
        let offset = (addr - 0x0600_0000) as usize;
        if offset < 0x4000 {
            let tile = offset / 32;
            let byte_in_tile = offset % 32;
            println!(
                "  addr={:#010X}(tile {} byte {}) pc={:#010X} val={:#04X}",
                addr, tile, byte_in_tile, pc, val
            );
            count += 1;
            if count >= 20 {
                break;
            }
        }
    }
}

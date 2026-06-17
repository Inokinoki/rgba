use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    gba.mem_mut().vram_log_enabled = true;

    for frame in 0..195u32 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    gba.mem_mut().vram_log_enabled = false;

    let log = &gba.mem().vram_write_log;

    let tile_writes: Vec<_> = log
        .iter()
        .filter(|(a, _, _)| *a >= 0x06000000 && *a < 0x0600C000)
        .collect();

    println!("Total tile-area writes: {}", tile_writes.len());

    let last_50: Vec<_> = tile_writes.iter().rev().take(50).rev().copied().collect();

    println!("\n=== Last 50 tile-area VRAM writes ===");
    for (addr, pc, val) in &last_50 {
        let tile = (addr - 0x06000000) / 32;
        let byte_in_tile = (addr - 0x06000000) % 32;
        println!(
            "  addr={:#010X} tile={}[{}] pc={:#010X} val={:#04X}",
            addr, tile, byte_in_tile, pc, val
        );
    }

    let max_addr = tile_writes.iter().map(|(a, _, _)| *a).max().unwrap_or(0);
    println!(
        "\nMax VRAM addr written in tile area: {:#010X} (tile {})",
        max_addr,
        (max_addr - 0x06000000) / 32
    );

    let vram = gba.mem().vram();
    let mut nonzero_tiles = 0;
    let mut last_nonzero_tile = 0u32;
    for tile in 0..1024u32 {
        let base = tile as usize * 32;
        let mut has_data = false;
        for b in 0..32 {
            if vram[base + b] != 0 {
                has_data = true;
                break;
            }
        }
        if has_data {
            nonzero_tiles += 1;
            last_nonzero_tile = tile;
        }
    }
    println!(
        "Nonzero tiles in 0x0000-0x7FFF: {} (last tile: {})",
        nonzero_tiles, last_nonzero_tile
    );

    println!("\n=== Map data sample (0xC000 base) ===");
    for i in 0..32u32 {
        let off = 0xC000 + (i as usize) * 2;
        let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
        let tile_num = entry & 0x3FF;
        let hflip = (entry >> 10) & 1;
        let vflip = (entry >> 11) & 1;
        let pal = (entry >> 12) & 0xF;
        if tile_num > 113 {
            println!(
                "  map[{}]: tile={} hflip={} vflip={} pal={} — REFERS TO EMPTY TILE!",
                i, tile_num, hflip, vflip, pal
            );
        }
    }

    let mut map_refs_empty = 0u32;
    let mut map_refs_total = 0u32;
    for i in 0..(0x4000 / 2) {
        let off = 0xC000 + i * 2;
        let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
        let tile_num = entry & 0x3FF;
        if tile_num != 0 {
            map_refs_total += 1;
            let tile_base = tile_num as usize * 32;
            let mut empty = true;
            for b in 0..32 {
                if vram[tile_base + b] != 0 {
                    empty = false;
                    break;
                }
            }
            if empty {
                map_refs_empty += 1;
            }
        }
    }
    println!(
        "\nMap entries referencing nonzero tiles: {} total, {} refer to empty tiles ({:.1}%)",
        map_refs_total,
        map_refs_empty,
        map_refs_empty as f64 / map_refs_total as f64 * 100.0
    );
}

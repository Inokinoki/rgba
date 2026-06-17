use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    println!("=== Single-frame VRAM write trace ===");
    println!("Running 200 frames to get past init...");
    for _ in 0..200u32 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    let vram_before = gba.mem().vram().to_vec();

    println!("Running 1 frame (frame 200)...");
    gba.run_frame_parallel(&mut framebuffer);

    let vram_after = gba.mem().vram().to_vec();

    let mut changed_bytes = 0;
    let mut changed_ranges: Vec<(usize, usize)> = Vec::new();
    let mut range_start: Option<usize> = None;

    for i in 0..vram_before.len() {
        if vram_before[i] != vram_after[i] {
            changed_bytes += 1;
            if range_start.is_none() {
                range_start = Some(i);
            }
        } else {
            if let Some(start) = range_start.take() {
                changed_ranges.push((start, i));
            }
        }
    }
    if let Some(start) = range_start.take() {
        changed_ranges.push((start, vram_before.len()));
    }

    println!(
        "Changed bytes in VRAM: {}/{}",
        changed_bytes,
        vram_before.len()
    );
    println!("Changed ranges: {}", changed_ranges.len());
    for (start, end) in changed_ranges.iter().take(30) {
        let size = end - start;
        println!("  {:#06X}..{:#06X} ({} bytes)", start, end, size);
    }

    let nonzero_before = vram_before.iter().filter(|&&b| b != 0).count();
    let nonzero_after = vram_after.iter().filter(|&&b| b != 0).count();
    println!(
        "Nonzero VRAM before: {} after: {} delta: {}",
        nonzero_before,
        nonzero_after,
        nonzero_after as i64 - nonzero_before as i64
    );

    println!("\n=== Checking if VRAM writes are happening at all ===");
    let vram = gba.mem().vram();
    let mut tile_data_bytes = 0;
    for i in 0..0x10000 {
        if vram[i] != 0 {
            tile_data_bytes += 1;
        }
    }
    let mut map_data_bytes = 0;
    for i in 0x10000..0x18000 {
        if vram[i] != 0 {
            map_data_bytes += 1;
        }
    }
    println!(
        "Tile data (0x0000-0xFFFF): {}/65536 ({:.1}%)",
        tile_data_bytes,
        tile_data_bytes as f64 / 65536.0 * 100.0
    );
    println!(
        "Map data (0x10000-0x17FFF): {}/32768 ({:.1}%)",
        map_data_bytes,
        map_data_bytes as f64 / 32768.0 * 100.0
    );

    println!("\n=== BG registers ===");
    let bgcnt: Vec<u16> = (0..4).map(|i| gba.ppu().get_bgcnt(i)).collect();
    let bghofs: Vec<u16> = (0..4).map(|i| gba.ppu().get_bg_hofs(i)).collect();
    let bgvofs: Vec<u16> = (0..4).map(|i| gba.ppu().get_bg_vofs(i)).collect();
    for i in 0..4 {
        let tile_base = ((bgcnt[i] as usize & 0x3) << 14);
        let map_base = ((bgcnt[i] as usize >> 8) & 0x1F) << 11;
        let size = (bgcnt[i] >> 14) & 3;
        let prio = bgcnt[i] & 3;
        let bpp4 = (bgcnt[i] & 0x80) == 0;
        println!(
            "BG{}: cnt={:#06X} pri={} tile={:#06X} map={:#06X} size={} {} hofs={} vofs={}",
            i,
            bgcnt[i],
            prio,
            tile_base,
            map_base,
            size,
            if bpp4 { "4bpp" } else { "8bpp" },
            bghofs[i],
            bgvofs[i]
        );
    }

    println!("\n=== Tile data analysis ===");
    let mut tiles_with_data = 0;
    let mut max_tile = 0;
    for tile_idx in 0..1024 {
        let base = tile_idx * 32;
        let has_data = (0..32).any(|j| vram[base + j] != 0);
        if has_data {
            tiles_with_data += 1;
            max_tile = tile_idx;
        }
    }
    println!(
        "Tiles 0-1023 with data: {} (max tile: {})",
        tiles_with_data, max_tile
    );

    let mut tiles_with_data_8bpp = 0;
    for tile_idx in 0..512 {
        let base = tile_idx * 64;
        if base + 64 <= vram.len() {
            let has_data = (0..64).any(|j| vram[base + j] != 0);
            if has_data {
                tiles_with_data_8bpp += 1;
            }
        }
    }
    println!("8bpp tiles 0-511 with data: {}", tiles_with_data_8bpp);

    println!("\n=== Map entries at 0xC000-0xFFFF (BG maps) ===");
    let mut nonzero_entries = 0;
    let mut tile_min = u16::MAX;
    let mut tile_max = 0u16;
    for i in (0xC000..0x10000).step_by(2) {
        let entry = u16::from_le_bytes([vram[i], vram[i + 1]]);
        if entry != 0 {
            nonzero_entries += 1;
            let tile = entry & 0x3FF;
            tile_min = tile_min.min(tile);
            tile_max = tile_max.max(tile);
        }
    }
    println!("Nonzero map entries: {}/8192", nonzero_entries);
    println!("Tile range in map entries: {}..={}", tile_min, tile_max);
    println!(
        "Max tile referenced: {} — but only {} tiles have data!",
        tile_max, tiles_with_data
    );
}

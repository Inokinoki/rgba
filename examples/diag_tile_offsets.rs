use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.sync_ppu_full();

    let vram = gba.mem.vram();

    // Check tiles around 394 to see if data is offset
    for tile_id in 390..400 {
        let offset = tile_id * 32;
        let mut all_zero = true;
        for b in 0..32 {
            if vram[offset + b] != 0 {
                all_zero = false;
                break;
            }
        }
        if !all_zero {
            let hex: String = (0..32)
                .map(|b| format!("{:02X}", vram[offset + b]))
                .collect::<Vec<_>>()
                .join(" ");
            eprintln!("Tile {}: {}", tile_id, hex);
        } else {
            eprintln!("Tile {}: ALL ZEROS", tile_id);
        }
    }

    // Also check around tile 403
    eprintln!("\nTiles around 403:");
    for tile_id in 400..410 {
        let offset = tile_id * 32;
        let mut all_zero = true;
        for b in 0..32 {
            if vram[offset + b] != 0 {
                all_zero = false;
                break;
            }
        }
        if !all_zero {
            let hex: String = (0..32)
                .map(|b| format!("{:02X}", vram[offset + b]))
                .collect::<Vec<_>>()
                .join(" ");
            eprintln!("Tile {}: {}", tile_id, hex);
        } else {
            eprintln!("Tile {}: ALL ZEROS", tile_id);
        }
    }

    // Count non-zero tiles in char_base area
    let mut nonzero_count = 0;
    let mut first_nonzero = 1024u16;
    let mut last_nonzero = 0u16;
    for tile_id in 0..1024u16 {
        let offset = tile_id as usize * 32;
        let all_zero = vram[offset..offset + 32].iter().all(|&b| b == 0);
        if !all_zero {
            nonzero_count += 1;
            first_nonzero = first_nonzero.min(tile_id);
            last_nonzero = last_nonzero.max(tile_id);
        }
    }
    eprintln!(
        "\nNon-zero tiles in BG VRAM: {}/1024 (first={}, last={})",
        nonzero_count, first_nonzero, last_nonzero
    );

    // Check if there's data in OBJ VRAM that should be in BG VRAM
    let obj_start = 0x10000;
    let mut obj_nonzero = 0;
    for i in 0..512 {
        let offset = obj_start + i * 32;
        if offset + 32 <= vram.len() {
            let all_zero = vram[offset..offset + 32].iter().all(|&b| b == 0);
            if !all_zero {
                obj_nonzero += 1;
            }
        }
    }
    eprintln!("Non-zero tiles in OBJ VRAM (0x10000): {}/512", obj_nonzero);

    // Check BG0 hofs and the visible tile range
    let hofs = gba.ppu.get_bg_hofs(0);
    let vofs = gba.ppu.get_bg_vofs(0);
    eprintln!("\nBG0 hofs={} vofs={}", hofs, vofs);
    let first_col = (hofs / 8) as usize;
    let first_row = (vofs / 8) as usize;
    eprintln!(
        "Visible tile range: cols {}-{}, rows {}-{}",
        first_col,
        first_col + 29,
        first_row,
        first_row + 19
    );
}

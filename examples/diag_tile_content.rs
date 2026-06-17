use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..600 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.sync_ppu_full();

    let vram = gba.mem.vram();

    // Check tiles used in visible BG0 area
    let tiles_to_check = [394, 403, 611, 482, 317, 318, 315, 473];
    for tile_id in tiles_to_check {
        let offset = tile_id * 32;
        let mut all_zero = true;
        for b in 0..32 {
            if vram[offset + b] != 0 {
                all_zero = false;
                break;
            }
        }
        if all_zero {
            eprintln!("Tile {}: ALL ZEROS", tile_id);
        } else {
            let hex: String = (0..32)
                .map(|b| format!("{:02X}", vram[offset + b]))
                .collect::<Vec<_>>()
                .join(" ");
            eprintln!("Tile {}: {}", tile_id, hex);
        }
    }

    // Check palette banks 0 and 11 (used in visible screen entries)
    for bank in [0u16, 11] {
        eprintln!("\nPalette bank {}:", bank);
        for i in 0..16 {
            let c = gba.mem.read_palette_color(0, bank * 16 + i);
            if c != 0 {
                let r = (c & 0x1F) as u32 * 255 / 31;
                let g = ((c >> 5) & 0x1F) as u32 * 255 / 31;
                let b = ((c >> 10) & 0x1F) as u32 * 255 / 31;
                eprintln!("  color[{:2}] = 0x{:04X} (R={} G={} B={})", i, c, r, g, b);
            }
        }
    }

    // Check screen entry layout for BG0 visible area (row 0)
    let bg0_screen = 0x0C000usize;
    eprintln!("\nBG0 screen entries row 0 (first 64 entries):");
    let mut tile_hist: std::collections::HashMap<u16, usize> = std::collections::HashMap::new();
    for i in 0..64 {
        let off = bg0_screen + i * 2;
        let se = vram[off] as u16 | ((vram[off + 1] as u16) << 8);
        let tile = se & 0x3FF;
        let pal = (se >> 12) & 0xF;
        *tile_hist.entry(tile).or_insert(0) += 1;
        if i < 32 {
            eprintln!("  [{:2}] tile={} pal={}", i, tile, pal);
        }
    }
    let mut sorted: Vec<_> = tile_hist.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    eprintln!(
        "Tile histogram (first 64): {:?}",
        sorted
            .iter()
            .take(10)
            .map(|(&t, &c)| (t, c))
            .collect::<Vec<_>>()
    );

    // Also check: does the rendering code see the same data?
    // Render pixel (0,0) manually
    let x = 0u16;
    let y = 0u16;
    let bg0cnt = gba.ppu.get_bgcnt(0);
    let hofs = gba.ppu.get_bg_hofs(0);
    let vofs = gba.ppu.get_bg_vofs(0);
    let bg_x = ((x as u32 + hofs as u32) % 512) as u16;
    let bg_y = ((y as u32 + vofs as u32) % 256) as u16;
    let tile_x = bg_x / 8;
    let tile_y = bg_y / 8;
    let pixel_x = (bg_x % 8) as u8;
    let pixel_y = (bg_y % 8) as u8;
    eprintln!(
        "\nPixel (0,0): bg=({},{}) tile=({},{}) pixel=({},{})",
        bg_x, bg_y, tile_x, tile_y, pixel_x, pixel_y
    );
}

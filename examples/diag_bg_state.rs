use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for i in 0..240 {
        gba.run_frame_parallel(&mut fb);
        if i == 120 || i == 239 {
            eprintln!("=== Frame {} ===", i + 1);
            let dispcnt = gba.mem.read_half(0x04000000);
            eprintln!("DISPCNT: 0x{:04X}", dispcnt);
            let bg_mode = dispcnt & 7;
            eprintln!("  BG mode: {}", bg_mode);
            eprintln!(
                "  BG0-3: {}{}{}{} OBJ: {}",
                (dispcnt >> 8) & 1,
                (dispcnt >> 9) & 1,
                (dispcnt >> 10) & 1,
                (dispcnt >> 11) & 1,
                (dispcnt >> 12) & 1
            );

            let mut bg_configs = Vec::new();
            for bg in 0..4 {
                let bgcnt = gba.mem.read_half(0x04000008 + bg * 2);
                let priority = bgcnt & 3;
                let char_base = ((bgcnt >> 2) & 3) as usize * 0x4000;
                let palette_mode = (bgcnt >> 7) & 1;
                let screen_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
                let size = (bgcnt >> 14) & 3;
                let bg_hofs = gba.mem.read_half(0x04000010 + bg * 4) & 0x1FF;
                let bg_vofs = gba.mem.read_half(0x04000012 + bg * 4) & 0x1FF;
                eprintln!(
                    "  BG{}: pri={} cb=0x{:05X} sb=0x{:05X} pal={} size={} hofs={} vofs={}",
                    bg, priority, char_base, screen_base, palette_mode, size, bg_hofs, bg_vofs
                );
                bg_configs.push((char_base, screen_base, palette_mode));
            }

            // Read VRAM snapshot
            let vram = gba.mem.vram().to_vec();

            // Dump screen entries for each BG
            for bg in 0..4 {
                let (char_base, screen_base, _) = bg_configs[bg];
                eprintln!("  BG{} screen entries at 0x{:05X}:", bg, screen_base);
                for i in 0..8 {
                    let off = screen_base + i * 2;
                    if off + 1 < vram.len() {
                        let se = vram[off] as u16 | (vram[off + 1] as u16) << 8;
                        let tile = se & 0x3FF;
                        let pal = (se >> 12) & 0xF;
                        eprintln!("    [{:2}] tile={} pal={}", i, tile, pal);
                    }
                }
                // Count tile usage
                let mut tile_hist: std::collections::HashMap<u16, usize> =
                    std::collections::HashMap::new();
                for i in 0..1024 {
                    let off = screen_base + i * 2;
                    if off + 1 < vram.len() {
                        let se = vram[off] as u16 | (vram[off + 1] as u16) << 8;
                        let tile = se & 0x3FF;
                        *tile_hist.entry(tile).or_insert(0) += 1;
                    }
                }
                let mut sorted: Vec<_> = tile_hist.iter().collect();
                sorted.sort_by(|a, b| b.1.cmp(a.1));
                eprintln!(
                    "    Top tiles: {:?}",
                    sorted
                        .iter()
                        .take(5)
                        .map(|(&t, &c)| (t, c))
                        .collect::<Vec<_>>()
                );
            }

            // Dump some tile data
            let char_base = bg_configs[0].0;
            for tile_id in [0usize, 1, 2, 100, 512, 1023] {
                let offset = char_base + tile_id * 32;
                if offset + 32 <= vram.len() {
                    let all_zero = vram[offset..offset + 32].iter().all(|&b| b == 0);
                    if !all_zero {
                        let hex: String = (0..16)
                            .map(|b| format!("{:02X}", vram[offset + b]))
                            .collect::<Vec<_>>()
                            .join(" ");
                        eprintln!("  Tile {} @0x{:05X}: {}", tile_id, offset, hex);
                    }
                }
            }

            // Palette info
            let backdrop = gba.mem.read_half(0x05000000);
            eprintln!("  Backdrop: 0x{:04X}", backdrop);
            let mut pal_nonzero = 0;
            for i in 0..256 {
                if gba.mem.read_half(0x05000000 + (i * 2) as u32) != 0 {
                    pal_nonzero += 1;
                }
            }
            eprintln!("  Non-zero palette entries: {}/256", pal_nonzero);
        }
    }
}

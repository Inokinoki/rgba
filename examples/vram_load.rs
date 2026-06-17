use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    gba.mem_mut().swi_log_enabled = true;

    let mut framebuffer = vec![0u32; 240 * 160];

    let mut prev_vram_nonzero = 0usize;
    let mut prev_dma_log_len = 0usize;

    for frame in 0..300u32 {
        gba.run_frame_parallel(&mut framebuffer);

        if frame % 20 != 0 && frame != 299 {
            continue;
        }

        let vram = gba.mem().vram();
        let vram_nonzero = vram.iter().filter(|&&b| b != 0).count();

        let dma_log = &gba.mem().dma_log;
        let new_dma_entries = if dma_log.len() > prev_dma_log_len {
            &dma_log[prev_dma_log_len..]
        } else {
            &[]
        };
        prev_dma_log_len = dma_log.len();

        let ppu = gba.ppu();
        let dispcnt = ppu.get_dispcnt();
        if dispcnt & 0x80 != 0 {
            println!(
                "Frame {}: FORCED BLANK, VRAM nonzero={}",
                frame, vram_nonzero
            );
            prev_vram_nonzero = vram_nonzero;
            continue;
        }

        let enabled_bits = (dispcnt >> 8) & 0xF;
        let mut unique = std::collections::HashSet::new();
        for &p in &framebuffer {
            unique.insert(p);
        }

        println!(
            "Frame {}: colors={} VRAM nonzero={} (+{} bytes) new_DMA={} total_DMA={}",
            frame,
            unique.len(),
            vram_nonzero,
            vram_nonzero - prev_vram_nonzero,
            new_dma_entries.len(),
            dma_log.len()
        );

        if !new_dma_entries.is_empty() && frame <= 260 {
            for (i, entry) in new_dma_entries.iter().enumerate().take(20) {
                println!(
                    "  DMA: ch={} src={:#010X} dst={:#010X} cnt={:#010X} ctrl={:#06X}",
                    entry.0, entry.1, entry.2, entry.3, entry.4
                );
            }
        }

        if frame >= 240 {
            for bg in 0..4usize {
                if !ppu.is_bg_enabled(bg) {
                    continue;
                }
                let bgcnt = ppu.get_bgcnt(bg);
                let tile_base = ((bgcnt >> 2) & 0x3) as usize * 0x4000;
                let map_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
                let hofs = ppu.get_bg_hofs(bg);
                let vofs = ppu.get_bg_vofs(bg);

                let vram = ppu.vram();
                let mut non_empty = 0u32;
                let mut total = 0u32;
                for ty in 0..20u16 {
                    for tx in 0..30u16 {
                        let bx = tx * 8 + hofs;
                        let by = ty * 8 + vofs;
                        let tile_x = bx / 8;
                        let tile_y = by / 8;
                        let entry_off = map_base + (tile_y as usize * 32 + tile_x as usize) * 2;
                        if entry_off + 1 >= vram.len() {
                            continue;
                        }
                        let entry = u16::from_le_bytes([vram[entry_off], vram[entry_off + 1]]);
                        let tile_num = entry & 0x3FF;
                        total += 1;
                        if tile_num == 0 || tile_num == 1023 {
                            continue;
                        }
                        let tile_off = tile_base + tile_num as usize * 32;
                        if tile_off + 32 > vram.len() {
                            continue;
                        }
                        if (0..32).any(|i| vram[tile_off + i] != 0) {
                            non_empty += 1;
                        }
                    }
                }
                println!(
                    "  BG{}: map={:#X} tile_base={:#X} hofs={} vofs={} non_empty={}/{}",
                    bg, map_base, tile_base, hofs, vofs, non_empty, total
                );
            }
        }

        prev_vram_nonzero = vram_nonzero;
    }
}

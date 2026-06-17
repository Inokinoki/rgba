use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    gba.mem_mut().vram_log_enabled = true;

    for frame in 0..260u32 {
        let prev_vram = gba.mem().vram().to_vec();

        gba.run_frame_parallel(&mut framebuffer);

        let cur_vram = gba.mem().vram().to_vec();

        let mut new_tile_count = 0u32;
        let mut new_tile_max = 0u32;
        for tile in 0..1024u32 {
            let base = tile as usize * 32;
            let mut was_empty = true;
            let mut is_now_filled = false;
            for b in 0..32 {
                if prev_vram[base + b] != 0 {
                    was_empty = false;
                }
                if cur_vram[base + b] != 0 {
                    is_now_filled = true;
                }
            }
            if was_empty && is_now_filled {
                new_tile_count += 1;
                new_tile_max = tile;
            }
        }

        if new_tile_count > 0 {
            let dispcnt_io = gba.mem().io();
            let dispcnt = u16::from_le_bytes([dispcnt_io[0], dispcnt_io[1]]);
            println!(
                "Frame {}: {} new tiles loaded (max={}), DISPCNT={:#06X}",
                frame, new_tile_count, new_tile_max, dispcnt
            );
        }

        let log = &gba.mem().vram_write_log;
        let tile_writes: Vec<_> = log
            .iter()
            .filter(|(a, _, _)| *a >= 0x06000000 && *a < 0x0600C000)
            .collect();
        if tile_writes.len() > 0 && frame > 5 {
            let max_addr = tile_writes.iter().map(|(a, _, _)| *a).max().unwrap_or(0);
            let max_tile = (max_addr - 0x06000000) / 32;
        }
    }

    gba.mem_mut().vram_log_enabled = false;

    let vram = gba.mem().vram();
    let mut total_nonzero = 0;
    for tile in 0..1024u32 {
        let base = tile as usize * 32;
        for b in 0..32 {
            if vram[base + b] != 0 {
                total_nonzero += 1;
                break;
            }
        }
    }
    println!("\nFinal: {} nonzero tiles in tile area", total_nonzero);

    let io = gba.mem().io();
    let dispcnt = u16::from_le_bytes([io[0], io[1]]);
    println!("DISPCNT: {:#06X}", dispcnt);
    for bg in 0..4u32 {
        let off = 0x08 + bg as usize * 2;
        let bgcnt = u16::from_le_bytes([io[off], io[off + 1]]);
        let enabled = (dispcnt >> (8 + bg)) & 1;
        if enabled != 0 || bgcnt != 0 {
            let char_base = ((bgcnt >> 2) & 3) as u32 * 0x4000;
            let screen_base = ((bgcnt >> 8) & 0x1F) as u32 * 0x800;
            let size = (bgcnt >> 14) & 3;
            let priority = bgcnt & 3;
            println!(
                "BG{}: priority={} char={:#X} screen={:#X} size={} enabled={}",
                bg, priority, char_base, screen_base, size, enabled
            );
        }
    }
}

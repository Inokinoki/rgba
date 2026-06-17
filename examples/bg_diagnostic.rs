use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    gba.mem_mut().swi_log_enabled = true;
    gba.mem_mut().dma_log_enabled = true;
    gba.mem_mut().cpu_set_log_enabled = true;

    for _ in 0..240 {
        gba.run_frame_parallel(&mut framebuffer);
    }
    gba.input_mut().press_key(KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut framebuffer);
    }
    gba.input_mut().release_key(KeyState::START);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut framebuffer);
    }
    for _ in 0..100 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..3 {
            gba.run_frame_parallel(&mut framebuffer);
        }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..12 {
            gba.run_frame_parallel(&mut framebuffer);
        }
    }

    println!("=== SWI calls made ===");
    let swi_log = &gba.mem().swi_log;
    let mut swi_counts: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    for &swi in swi_log {
        *swi_counts.entry(swi).or_insert(0) += 1;
    }
    let mut swi_sorted: Vec<_> = swi_counts.iter().collect();
    swi_sorted.sort_by_key(|(k, _)| **k);
    for (&swi, &count) in swi_sorted {
        println!("  SWI 0x{:02X}: {} calls", swi, count);
    }

    println!("\n=== CpuSet calls to VRAM/Palette ===");
    let cpu_set_log = &gba.mem().cpu_set_log;
    let mut idx = 0;
    for &(src, dst, cnt) in cpu_set_log.iter() {
        let fill = (cnt >> 24) & 1 != 0;
        let num = cnt & 0x1FFFFF;
        let is_32 = (cnt >> 26) & 1 != 0;
        let dst_in_vram = dst >= 0x0600_0000 && dst < 0x0602_0000;
        let dst_in_palette = dst >= 0x0500_0000 && dst < 0x0500_0400;
        if dst_in_vram || dst_in_palette {
            println!(
                "  [{}] src={:#010X} dst={:#010X} cnt={:#010X} fill={} is32={} count={} {}",
                idx,
                src,
                dst,
                cnt,
                fill,
                is_32,
                num,
                if dst_in_vram { "VRAM" } else { "PALETTE" }
            );
            idx += 1;
            if idx >= 50 {
                break;
            }
        }
    }

    println!("\n=== Last 30 DMA transfers to VRAM ===");
    let dma_log = &gba.mem().dma_log;
    let vram_dmas: Vec<_> = dma_log
        .iter()
        .filter(|&&(ch, _, dst, _, _)| dst >= 0x0600_0000 && dst < 0x0602_0000)
        .collect();
    for (i, &&(ch, src, dst, num, size)) in vram_dmas.iter().rev().take(30).enumerate() {
        println!(
            "  [{}] DMA{} src={:#010X} dst={:#010X} count={} size={}",
            i, ch, src, dst, num, size
        );
    }

    let dispcnt = gba.mem_mut().read_half(0x0400_0000);
    let bg3cnt = gba.mem_mut().read_half(0x0400_000E);
    let bg3hofs = gba.mem_mut().read_half(0x0400_0014) & 0x1FF;
    let bg3vofs = gba.mem_mut().read_half(0x0400_0016) & 0x1FF;

    let scr_base = ((bg3cnt >> 8) & 0x1F) as u32 * 0x800;
    let tile_base = ((bg3cnt >> 2) & 0x3) as u32 * 0x4000;

    println!("\n=== BG3 Config ===");
    println!("  DISPCNT={:#06X} BG3CNT={:#06X}", dispcnt, bg3cnt);
    println!(
        "  scr_base={:#010X} tile_base={:#010X}",
        scr_base, tile_base
    );
    println!("  BG3HOFS={} BG3VOFS={}", bg3hofs, bg3vofs);

    let vram = gba.mem().vram();
    let palette = gba.mem().palette();

    println!("\n=== BG3 screen entries ===");
    for ty in 0..4u32 {
        for tx in 0..8u32 {
            let map_x = (tx * 8 + bg3hofs as u32) / 8;
            let map_y = (ty * 8 + bg3vofs as u32) / 8;
            let screen_block = (map_y / 32) * 32 + (map_x / 32);
            let entry_offset =
                scr_base + screen_block * 0x800 + ((map_y % 32) * 32 + (map_x % 32)) * 2;
            let entry =
                u16::from_le_bytes([vram[entry_offset as usize], vram[entry_offset as usize + 1]]);
            let tile_num = entry & 0x3FF;
            let pal = (entry >> 12) & 0xF;
            print!(" ({},{})T{}P{}", tx, ty, tile_num, pal);
        }
        println!();
    }

    println!(
        "\n=== Tile 279 (grass) at offset {:#X} ===",
        tile_base + 279 * 32
    );
    let tile_offset = (tile_base + 279 * 32) as usize;
    print!("  Bytes:");
    for i in 0..32 {
        if i % 16 == 0 {
            print!("\n    ");
        }
        print!("{:02X} ", vram[tile_offset + i]);
    }
    println!();

    println!("\n=== Palette 4 ===");
    for i in 0..16 {
        let off = (4 * 16 + i) * 2;
        let color = u16::from_le_bytes([palette[off], palette[off + 1]]);
        if color != 0 {
            let r = color & 0x1F;
            let g = (color >> 5) & 0x1F;
            let b = (color >> 10) & 0x1F;
            println!("  [{}] {:#06X} R={} G={} B={}", i, color, r, g, b);
        }
    }

    println!("\n=== Non-zero BG palettes ===");
    for pal in 0..16 {
        let mut non_zero = 0;
        for i in 0..16 {
            let off = (pal * 16 + i) * 2;
            let color = u16::from_le_bytes([palette[off], palette[off + 1]]);
            if color != 0 {
                non_zero += 1;
            }
        }
        if non_zero > 0 {
            println!("  Palette {}: {}/16 non-zero", pal, non_zero);
        }
    }

    println!("\n=== VRAM usage by region ===");
    let regions = [
        ("Tiles 0 (0x0000-0x3FFF)", 0x0000usize, 0x4000usize),
        ("Tiles 1 (0x4000-0x7FFF)", 0x4000, 0x8000),
        ("Tiles 2 (0x8000-0xBFFF)", 0x8000, 0xC000),
        ("Tiles 3 (0xC000-0xFFFF)", 0xC000, 0x10000),
        ("SB 0x10 (0x10000-0x107FF)", 0x10000, 0x10800),
        ("SB 0x1C (0x1C000-0x1C7FF)", 0x1C000, 0x1C800),
        ("SB 0x1E (0x1E000-0x1E7FF)", 0x1E000, 0x1E800),
    ];
    for (name, start, end) in &regions {
        let mut nz = 0u32;
        for i in *start..*end {
            if vram[i] != 0 {
                nz += 1;
            }
        }
        if nz > 0 {
            println!(
                "  {}: {}/{} ({:.1}%)",
                name,
                nz,
                end - start,
                nz as f64 / (end - start) as f64 * 100.0
            );
        }
    }

    println!(
        "\n=== First 20 non-zero tiles at tile_base={:#X} ===",
        tile_base
    );
    let mut found = 0u32;
    for t in 0..1024u32 {
        let off = (tile_base + t * 32) as usize;
        let mut has_data = false;
        for i in 0..32 {
            if off + i < vram.len() && vram[off + i] != 0 {
                has_data = true;
                break;
            }
        }
        if has_data {
            print!("  Tile {}: ", t);
            for i in 0..8 {
                print!("{:02X}", vram[off + i]);
            }
            println!();
            found += 1;
            if found >= 20 {
                break;
            }
        }
    }
}

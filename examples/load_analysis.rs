use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    // Enable all logging
    gba.mem_mut().dma_log_enabled = true;
    gba.mem_mut().dma_log.clear();
    gba.mem_mut().swi_log_enabled = true;
    gba.mem_mut().swi_log.clear();
    gba.mem_mut().cpu_set_log_enabled = true;
    gba.mem_mut().cpu_set_log.clear();
    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();

    for frame in 0..300 {
        for _ in 0..228 {
            gba.run_scanline();
        }

        if frame == 220 {
            let dma_log = &gba.mem().dma_log;
            println!("=== DMA log at frame {} ===", frame);
            for &(ch, src, dst, cnt, ctrl) in dma_log.iter().take(50) {
                let dst_name = if dst >= 0x06010000 && dst < 0x06018000 {
                    "OBJ"
                } else if dst >= 0x06000000 && dst < 0x06010000 {
                    let off = dst - 0x06000000;
                    if off >= 0xF000 {
                        "ScreenBlock"
                    } else {
                        "BG_VRAM"
                    }
                } else if dst >= 0x02000000 && dst < 0x03000000 {
                    "EWRAM"
                } else if dst >= 0x03000000 && dst < 0x04000000 {
                    "IWRAM"
                } else {
                    "Other"
                };
                println!(
                    "  DMA{}: {:08X} -> {:08X} ({} words, ctrl={:08X}) [{}]",
                    ch, src, dst, cnt, ctrl, dst_name
                );
            }
            println!("  ... total DMA transfers: {}", dma_log.len());

            let swi_log = &gba.mem().swi_log;
            println!("\n=== SWI calls at frame {} ===", frame);
            let mut counts: std::collections::HashMap<u32, usize> =
                std::collections::HashMap::new();
            for &swi in swi_log {
                *counts.entry(swi).or_insert(0) += 1;
            }
            let mut sorted: Vec<_> = counts.iter().collect();
            sorted.sort_by_key(|(k, _)| **k);
            for (swi, count) in sorted {
                let name = match swi {
                    0x01 => "RegisterRamReset",
                    0x04 => "IntrWait",
                    0x06 => "Div",
                    0x08 => "Sqrt",
                    0x0A => "ArcTan2",
                    0x0B => "CpuSet",
                    0x0C => "CpuFastSet",
                    0x0D => "GetBiosChecksum",
                    0x0E => "BgAffineSet",
                    0x0F => "ObjAffineSet",
                    0x10 => "LZ77UncompWram",
                    0x11 => "LZ77UncompVram",
                    0x12 => "HuffUncomp",
                    0x13 => "RLUncompWram",
                    0x14 => "RLUncompVram",
                    0x15 => "DiffUncompWram",
                    0x16 => "DiffUncompVram",
                    0x1F => "VBlankIntrWait",
                    _ => "Unknown",
                };
                println!("  SWI 0x{:02X} ({}): {} calls", swi, name, count);
            }

            let cpu_set = &gba.mem().cpu_set_log;
            println!("\n=== CpuSet calls at frame {} ===", frame);
            for &(src, dst, cnt) in cpu_set.iter() {
                let dst_name = if dst >= 0x06000000 && dst < 0x06018000 {
                    "VRAM"
                } else if dst >= 0x02000000 && dst < 0x03000000 {
                    "EWRAM"
                } else if dst >= 0x03000000 && dst < 0x04000000 {
                    "IWRAM"
                } else {
                    "Other"
                };
                let src_name = if src >= 0x08000000 {
                    "ROM"
                } else if src >= 0x02000000 {
                    "EWRAM"
                } else if src >= 0x03000000 {
                    "IWRAM"
                } else {
                    "Other"
                };
                println!(
                    "  CpuSet: {} {:08X} -> {:08X} ({} words) [{}->{}]",
                    if cnt & 0x01000000 != 0 {
                        "FILL"
                    } else {
                        "COPY"
                    },
                    src,
                    dst,
                    cnt & 0xFFFFFF,
                    src_name,
                    dst_name
                );
            }
        }
    }
}

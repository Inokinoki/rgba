use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.dma_log_enabled = true;

    for _ in 0..300 {
        gba.run_frame_parallel(&mut fb);
    }

    // Check ALL DMA transfers
    let dma_log = &gba.mem.dma_log;
    eprintln!("Total DMA transfers: {}", dma_log.len());

    // Find ANY DMA that touches VRAM 0xC000-0xD000
    eprintln!("\nDMA transfers to VRAM 0xC000-0xD000:");
    let mut found = 0;
    for &(num, src, dst, count, size) in dma_log {
        if dst >= 0x0600_0000 && dst < 0x0602_0000 {
            let raw_offset = ((dst - 0x0600_0000) % 0x2_0000) as usize;
            let offset = if raw_offset >= 0x1_8000 {
                raw_offset - 0x8000
            } else {
                raw_offset
            };
            if offset >= 0xC000 && offset < 0xD000 {
                found += 1;
                eprintln!(
                    "  DMA{}: src=0x{:08X} dst=0x{:08X} count={} size={}",
                    num, src, dst, count, size
                );
            }
        }
    }
    eprintln!("Found: {} DMA transfers to screen entry area", found);

    // Also check DMA that touches 0xC000-0xFFFF
    eprintln!("\nDMA transfers to VRAM 0xC000-0xFFFF:");
    for &(num, src, dst, count, size) in dma_log {
        if dst >= 0x0600_0000 && dst < 0x0602_0000 {
            let raw_offset = ((dst - 0x0600_0000) % 0x2_0000) as usize;
            let offset = if raw_offset >= 0x1_8000 {
                raw_offset - 0x8000
            } else {
                raw_offset
            };
            if offset >= 0xC000 && offset < 0x10000 {
                eprintln!(
                    "  DMA{}: src=0x{:08X} dst=0x{:08X} count={} size={}",
                    num, src, dst, count, size
                );
            }
        }
    }

    // Check if 0x018A appears in IWRAM (DMA source)
    let iwram = gba.mem.iwram();
    eprintln!("\nSearching for 0x018A pattern in IWRAM...");
    let pattern = [0x8A, 0x01]; // little-endian 0x018A
    for i in 0..iwram.len() - 1 {
        if iwram[i] == pattern[0] && iwram[i + 1] == pattern[1] {
            eprintln!(
                "  Found at IWRAM offset 0x{:04X} (addr 0x{:08X})",
                i,
                0x03000000 + i
            );
        }
    }

    // Check if 0xB1DA pattern exists in IWRAM
    eprintln!("\nSearching for 0xB1DA pattern in IWRAM...");
    let pattern2 = [0xDA, 0xB1];
    for i in 0..iwram.len() - 1 {
        if iwram[i] == pattern2[0] && iwram[i + 1] == pattern2[1] {
            eprintln!(
                "  Found at IWRAM offset 0x{:04X} (addr 0x{:08X})",
                i,
                0x03000000 + i
            );
        }
    }

    // Check EWRAM too
    let wram = gba.mem.wram();
    eprintln!("\nSearching for 0xB1DA pattern in EWRAM...");
    let mut found_count = 0;
    for i in 0..wram.len() - 1 {
        if wram[i] == pattern2[0] && wram[i + 1] == pattern2[1] {
            found_count += 1;
            if found_count <= 10 {
                eprintln!(
                    "  Found at EWRAM offset 0x{:05X} (addr 0x{:08X})",
                    i,
                    0x02000000 + i
                );
            }
        }
    }
    eprintln!("  Total 0xB1DA occurrences in EWRAM: {}", found_count);

    // Check what value 0x018A appears in EWRAM
    eprintln!("\nSearching for 0x018A pattern in EWRAM...");
    found_count = 0;
    for i in 0..wram.len() - 1 {
        if wram[i] == pattern[0] && wram[i + 1] == pattern[1] {
            found_count += 1;
            if found_count <= 10 {
                eprintln!(
                    "  Found at EWRAM offset 0x{:05X} (addr 0x{:08X})",
                    i,
                    0x02000000 + i
                );
            }
        }
    }
    eprintln!("  Total 0x018A occurrences in EWRAM: {}", found_count);
}

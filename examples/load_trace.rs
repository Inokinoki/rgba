use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    gba.mem_mut().vram_log_enabled = true;

    let mut framebuffer = vec![0u32; 240 * 160];

    for frame in 0..254 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    gba.mem_mut().vram_write_log.clear();
    let before_count = gba.mem().vram().iter().filter(|&&b| b != 0).count();

    gba.run_frame_parallel(&mut framebuffer);
    let after_count = gba.mem().vram().iter().filter(|&&b| b != 0).count();
    let writes = &gba.mem().vram_write_log;

    println!(
        "Frame 254: VRAM {} -> {} ({:+}), {} writes",
        before_count,
        after_count,
        after_count as i64 - before_count as i64,
        writes.len()
    );

    if !writes.is_empty() {
        let mut min_addr = u32::MAX;
        let mut max_addr = 0u32;
        let mut vr_writes = 0;
        for &(addr, _, _) in writes {
            let off = (addr as usize) & 0x1FFFF;
            if off < 0x18000 {
                min_addr = min_addr.min(off as u32);
                max_addr = max_addr.max(off as u32);
            }
            if addr >= 0x06000000 && addr < 0x06018000 {
                vr_writes += 1;
            }
        }
        println!("  VRAM write offsets: {:#X} - {:#X}", min_addr, max_addr);
        println!("  VRAM-mapped writes: {}", vr_writes);
    }

    gba.mem_mut().vram_write_log.clear();
    let before2 = after_count;
    gba.run_frame_parallel(&mut framebuffer);
    let after2 = gba.mem().vram().iter().filter(|&&b| b != 0).count();
    let writes2 = &gba.mem().vram_write_log;

    println!(
        "\nFrame 255: VRAM {} -> {} ({:+}), {} writes",
        before2,
        after2,
        after2 as i64 - before2 as i64,
        writes2.len()
    );

    for _ in 0..10 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    let final_count = gba.mem().vram().iter().filter(|&&b| b != 0).count();
    println!(
        "\nAfter 10 more frames: VRAM {} (delta from 255: {})",
        final_count,
        final_count as i64 - after2 as i64
    );

    let vram = gba.mem().vram();
    println!("\n=== Tiles 390-400 at base 0x0000 (referenced by BG0 map) ===");
    for tile in 390..401u32 {
        let off = (tile * 32) as usize;
        let mut nonzero = 0;
        for i in 0..32usize {
            if vram[off + i] != 0 { nonzero += 1; }
        }
        if nonzero > 0 {
            print!("  tile {} ({:#X}): ", tile, off);
            for i in 0..8usize { print!("{:02X}", vram[off + i]); }
            println!(" ({} nonzero)", nonzero);
        } else {
            println!("  tile {} ({:#X}): ALL ZEROS", tile, off);
        }
    }

    println!("\n=== First 20 tiles with data at base 0x0000 ===");
    let mut count = 0;
    for tile in 0..1200u32 {
        let off = (tile * 32) as usize;
        if off + 32 > vram.len() { break; }
        let nonzero = (0..32usize).filter(|&i| vram[off + i] != 0).count();
        if nonzero > 0 {
            println!("  tile {} ({:#06X}): {} nonzero bytes", tile, off, nonzero);
            count += 1;
            if count >= 20 { break; }
        }
    }

    println!("\n=== Last tile with data at base 0x0000 ===");
    for tile in (0..2000u32).rev() {
        let off = (tile * 32) as usize;
        if off + 32 > vram.len() { continue; }
        let nonzero = (0..32usize).filter(|&i| vram[off + i] != 0).count();
        if nonzero > 0 {
            println!("  tile {} ({:#06X}): {} nonzero bytes", tile, off, nonzero);
            break;
        }
    }
        }
        if nonzero > 0 {
            print!("  tile {} ({:#X}): ", tile, off);
            for i in 0..8 {
                print!("{:02X}", vram[off + i]);
            }
            println!(" ({} nonzero)", nonzero);
        } else {
            println!("  tile {} ({:#X}): ALL ZEROS", tile, off);
        }
    }

    println!("\n=== First 20 tiles with data at base 0x0000 ===");
    let mut count = 0;
    for tile in 0..1200u32 {
        let off = tile * 32;
        if off + 32 > vram.len() {
            break;
        }
        let nonzero = (0..32).filter(|&i| vram[off + i] != 0).count();
        if nonzero > 0 {
            println!("  tile {} ({:#06X}): {} nonzero bytes", tile, off, nonzero);
            count += 1;
            if count >= 20 {
                break;
            }
        }
    }

    println!("\n=== Last tile with data at base 0x0000 ===");
    for tile in (0..2000u32).rev() {
        let off = tile * 32;
        if off + 32 > vram.len() {
            continue;
        }
        let nonzero = (0..32).filter(|&i| vram[off + i] != 0).count();
        if nonzero > 0 {
            println!("  tile {} ({:#06X}): {} nonzero bytes", tile, off, nonzero);
            break;
        }
    }

    println!("\n=== VRAM write log during frame 254 (first 30 and last 30) ===");
    for (addr, pc, val) in writes.iter().take(30) {
        let offset = (*addr as usize) & 0x1FFFF;
        if *addr >= 0x06000000 && *addr < 0x06018000 {
            println!(
                "  VRAM+{:#06X} pc={:#010X} val={:#04X}",
                offset,
                pc * 2,
                val
            );
        }
    }
    if writes.len() > 60 {
        println!("  ... ({} more) ...", writes.len() - 60);
    }
    for (addr, pc, val) in writes.iter().rev().take(30) {
        let offset = (*addr as usize) & 0x1FFFF;
        if *addr >= 0x06000000 && *addr < 0x06018000 {
            println!(
                "  VRAM+{:#06X} pc={:#010X} val={:#04X}",
                offset,
                pc * 2,
                val
            );
        }
    }
}

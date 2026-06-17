use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.swi_log_enabled = true;
    gba.mem.cpu_set_log_enabled = true;

    for _ in 0..210 {
        gba.run_frame_parallel(&mut fb);
    }

    eprintln!(
        "SWI calls after 210 frames: {} total",
        gba.mem.swi_log.len()
    );
    let mut swi_counts: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    for &swi in &gba.mem.swi_log {
        *swi_counts.entry(swi).or_insert(0) += 1;
    }
    for (num, count) in &swi_counts {
        let name = match num {
            0x02 => "Halt",
            0x03 => "Stop",
            0x04 => "IntrWait",
            0x05 => "VBlankIntrWait",
            0x06 => "Div",
            0x07 => "DivArm",
            0x08 => "Sqrt",
            0x0B => "CpuSet",
            0x0C => "CpuFastSet",
            0x10 => "BitUnPack/LZ77",
            0x11 => "LZ77W16",
            0x12 => "LZ77Callback",
            0x13 => "HuffUnComp",
            0x14 => "HuffCallback",
            0x15 => "RLW16",
            _ => "Other",
        };
        eprintln!("  SWI 0x{:02X} ({}): {} calls", num, name, count);
    }

    eprintln!("\nCPU-set calls: {} total", gba.mem.cpu_set_log.len());
    for (i, &(src, dst, cnt)) in gba.mem.cpu_set_log.iter().enumerate() {
        let fill = (cnt >> 24) & 1;
        let count = cnt & 0x1FFFFF;
        let is_32 = (cnt >> 26) & 1;
        let dst_name = if dst >= 0x06000000 && dst < 0x06018000 {
            "VRAM"
        } else if dst >= 0x02000000 && dst < 0x02040000 {
            "EWRAM"
        } else if dst >= 0x03000000 && dst < 0x03008000 {
            "IWRAM"
        } else {
            "OTHER"
        };
        if i < 50 || dst_name == "VRAM" {
            eprintln!(
                "  #{}: src=0x{:08X} dst=0x{:08X}({}) cnt=0x{:08X} fill={} is32={} count={}",
                i, src, dst, dst_name, cnt, fill, is_32, count
            );
        }
    }

    // Now watch the transition from 90 to 344 tiles
    gba.mem.swi_log.clear();
    gba.mem.cpu_set_log.clear();

    eprintln!("\n=== Watching transition ===");
    for f in 0..20 {
        gba.run_frame_parallel(&mut fb);
        gba.sync_ppu_full();
        let vram = gba.mem.vram();
        let mut n = 0;
        for tid in 0..=343u32 {
            let off = tid as usize * 32;
            if vram[off..off + 32].iter().any(|&b| b != 0) {
                n += 1;
            }
        }
        eprintln!(
            "Frame 210+{}: tiles 0-343={}, swi_count={}",
            f,
            n,
            gba.mem.swi_log.len()
        );
        if n > 90 {
            // Show what SWI calls happened
            eprintln!("  SWI calls during this transition:");
            let mut tc: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
            for &swi in &gba.mem.swi_log {
                *tc.entry(swi).or_insert(0) += 1;
            }
            for (num, count) in &tc {
                eprintln!("    SWI 0x{:02X}: {} calls", num, count);
            }
            eprintln!("  CPU-set calls:");
            for (i, &(src, dst, cnt)) in gba.mem.cpu_set_log.iter().enumerate() {
                eprintln!(
                    "    #{}: src=0x{:08X} dst=0x{:08X} cnt=0x{:08X}",
                    i, src, dst, cnt
                );
            }
            break;
        }
    }
}

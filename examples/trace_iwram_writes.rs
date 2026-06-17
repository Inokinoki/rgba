use rgba::Gba;
use std::collections::BTreeMap;

const IWRAM_START: u32 = 0x03006DD8;
const IWRAM_END: u32 = 0x03006F58;
const NUM_ENTRIES: usize = 512;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem_mut().iwram_write_log_enabled = true;
    gba.mem_mut().iwram_write_log.clear();

    for frame in 0..240 {
        gba.run_frame_parallel(&mut fb);
        if (frame + 1) % 60 == 0 {
            eprintln!(
                "Frame {} done, log size: {}",
                frame + 1,
                gba.mem().iwram_write_log.len()
            );
        }
    }

    let iwram = gba.mem().iwram();
    let start_off = (IWRAM_START - 0x03000000) as usize;

    eprintln!("\n=== IWRAM screen entry dump at 0x{:08X} ===", IWRAM_START);
    let mut pal_counts: [usize; 16] = [0; 16];
    for i in 0..NUM_ENTRIES {
        let off = start_off + i * 2;
        let entry = u16::from_le_bytes([iwram[off], iwram[off + 1]]);
        let tile = entry & 0x3FF;
        let pal = (entry >> 12) & 0xF;
        pal_counts[pal as usize] += 1;
        if i % 32 == 0 {
            eprint!("\n  [{:03X}]: ", i);
        }
        if pal == 11 {
            eprint!("{:4} ", tile);
        } else {
            eprint!("P{}{:3} ", pal, tile);
        }
    }
    eprintln!();

    eprintln!("\nPalette distribution:");
    for (p, &count) in pal_counts.iter().enumerate() {
        if count > 0 {
            eprintln!("  palette {}: {} entries", p, count);
        }
    }

    eprintln!("\n=== IWRAM write log analysis ===");
    let log = &gba.mem().iwram_write_log;
    eprintln!("Total writes to range: {}", log.len());

    let mut pc_groups: BTreeMap<u32, Vec<(u32, u8)>> = BTreeMap::new();
    for &(addr, pc, val) in log {
        pc_groups.entry(pc).or_default().push((addr, val));
    }

    eprintln!("\nUnique write PCs:");
    for (pc, writes) in &pc_groups {
        let addrs: Vec<u32> = writes.iter().map(|(a, _)| *a).collect();
        let min_addr = addrs.iter().min().unwrap();
        let max_addr = addrs.iter().max().unwrap();
        let unique_vals: std::collections::HashSet<u8> = writes.iter().map(|(_, v)| *v).collect();
        let is_thumb = pc & 1 != 0;
        let real_pc = pc & !1;
        eprintln!(
            "  PC={:08X}{}: {} writes, addr {:08X}-{:08X}, {} unique byte values",
            real_pc,
            if is_thumb { " (Thumb)" } else { "" },
            writes.len(),
            min_addr,
            max_addr,
            unique_vals.len(),
        );

        // Reconstruct halfword writes for sample
        let mut hw_writes: BTreeMap<u32, Vec<u16>> = BTreeMap::new();
        let mut byte_map: BTreeMap<u32, [Option<u8>; 2]> = BTreeMap::new();
        for &(addr, val) in writes {
            let base = addr & !1;
            let byte_idx = (addr & 1) as usize;
            let entry = byte_map.entry(base).or_insert([None; 2]);
            entry[byte_idx] = Some(val);
            if entry[0].is_some() && entry[1].is_some() {
                let hw = u16::from_le_bytes([entry[0].unwrap(), entry[1].unwrap()]);
                hw_writes.entry(base).or_default().push(hw);
                *entry = [None; 2];
            }
        }

        let mut pal_histogram: [usize; 16] = [0; 16];
        for (_, hws) in &hw_writes {
            for &hw in hws {
                let pal = (hw >> 12) & 0xF;
                pal_histogram[pal as usize] += 1;
            }
        }
        let has_pal = pal_histogram.iter().any(|&c| c > 0);
        if has_pal {
            eprintln!("    Halfword palette distribution:");
            for (p, &count) in pal_histogram.iter().enumerate() {
                if count > 0 {
                    eprintln!("      palette {}: {} writes", p, count);
                }
            }
        }

        // Sample halfword values
        let sample_count = 5.min(hw_writes.len());
        eprintln!("    Sample halfword writes (first {}):", sample_count);
        for (addr, hws) in hw_writes.iter().take(sample_count) {
            for hw in hws.iter().take(3) {
                let tile = hw & 0x3FF;
                let pal = (hw >> 12) & 0xF;
                eprintln!(
                    "      {:08X} = {:04X} (tile={}, pal={})",
                    addr, hw, tile, pal
                );
            }
        }
    }

    // Also dump raw bytes for comparison with mGBA
    eprintln!("\n=== Raw hex dump for mGBA comparison ===");
    for i in 0..NUM_ENTRIES {
        let off = start_off + i * 2;
        let entry = u16::from_le_bytes([iwram[off], iwram[off + 1]]);
        if i % 16 == 0 {
            eprint!("\n{:04X}: ", IWRAM_START - 0x03000000 + (i * 2) as u32);
        }
        eprint!("{:04X} ", entry);
    }
    eprintln!();
}

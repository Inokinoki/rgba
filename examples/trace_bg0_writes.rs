use rgba::Gba;

const TRACE_BASE: u32 = 0x0600C000;
const TRACE_END: u32 = 0x0600C100;
const ROM_PATH: &str = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path(ROM_PATH).unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();

    for frame in 0..240 {
        gba.run_frame_parallel(&mut fb);
        if frame % 60 == 0 {
            eprintln!("Frame {}...", frame);
        }
    }

    let log = &gba.mem().vram_write_log;
    eprintln!("Total VRAM write log entries: {}", log.len());

    let target_writes: Vec<_> = log
        .iter()
        .filter(|(addr, _, _)| *addr >= TRACE_BASE && *addr < TRACE_END)
        .collect();
    eprintln!(
        "Writes to 0x{:08X}-0x{:08X}: {}",
        TRACE_BASE,
        TRACE_END,
        target_writes.len()
    );

    // Reconstruct halfword writes from consecutive byte pairs
    // write_half(addr, val) calls write_byte_internal(addr, lo) then write_byte_internal(addr+1, hi)
    // Each log entry is (addr, pc, byte_val)
    let mut halfword_writes: Vec<(u32, u16, u32)> = Vec::new();
    let mut i = 0;
    let entries: Vec<(u32, u32, u8)> = target_writes.iter().map(|&&(a, p, v)| (a, p, v)).collect();
    while i + 1 < entries.len() {
        let (addr0, pc0, val0) = entries[i];
        let (addr1, pc1, val1) = entries[i + 1];
        if addr1 == addr0 + 1 && pc0 == pc1 {
            let halfword = val0 as u16 | ((val1 as u16) << 8);
            halfword_writes.push((addr0, halfword, pc0));
            i += 2;
        } else {
            halfword_writes.push((addr0, val0 as u16, pc0));
            i += 1;
        }
    }
    if i < entries.len() {
        halfword_writes.push((entries[i].0, entries[i].2 as u16, entries[i].1));
    }

    // Group writes by target halfword address (aligned)
    use std::collections::BTreeMap;
    let mut writes_by_addr: BTreeMap<u32, Vec<(u16, u32)>> = BTreeMap::new();
    for (addr, val, pc) in &halfword_writes {
        let aligned = addr & !1;
        writes_by_addr.entry(aligned).or_default().push((*val, *pc));
    }

    println!("\n=== Halfword writes to BG0 screen entries (0x0600C000-0x0600C080) ===");
    println!(
        "{:<14} {:<8} {:<12} {:<10} {}",
        "Address", "Value", "PC", "Tile", "Pal"
    );
    println!("{}", "-".repeat(70));

    for entry_idx in 0..64u32 {
        let addr = TRACE_BASE + entry_idx * 2;
        if let Some(writes) = writes_by_addr.get(&addr) {
            let last = writes.last().unwrap();
            let tile = last.0 & 0x3FF;
            let pal = (last.0 >> 12) & 0xF;
            let is_border = entry_idx == 0 || entry_idx == 31 || entry_idx == 32 || entry_idx == 63;
            let marker = if is_border { " [BORDER]" } else { "" };

            // Show ALL writes to this address
            for (wi, (val, pc)) in writes.iter().enumerate() {
                let t = val & 0x3FF;
                let p = (val >> 12) & 0xF;
                let prefix = if wi == 0 {
                    format!("{:08X}", addr)
                } else {
                    "  (overwrite)".to_string()
                };
                println!(
                    "{:<14} {:04X}     {:08X}   {:>4} (p{}){}",
                    prefix,
                    val,
                    pc,
                    t,
                    p,
                    if wi == writes.len() - 1 { marker } else { "" }
                );
            }
        } else {
            println!("{:08X}     ----     --------    (no write)", addr);
        }
    }

    // Print final screen entries for comparison
    println!("\n=== Final screen entries at 0x0600C000 (read from VRAM) ===");
    println!(
        "{:<6} {:<14} {:<8} {:<6} {:<5} {}",
        "Idx", "Address", "Value", "Tile", "Pal", "Type"
    );
    println!("{}", "-".repeat(55));

    let vram = gba.mem().vram();
    let vram_base = (TRACE_BASE - 0x06000000) as usize;
    for i in 0..64u32 {
        let voff = vram_base + (i * 2) as usize;
        let val = u16::from_le_bytes([vram[voff], vram[voff + 1]]);
        let tile = val & 0x3FF;
        let pal = (val >> 12) & 0xF;
        let is_border = i == 0 || i == 31 || i == 32 || i == 63;
        let typ = if is_border { "BORDER" } else { "INTERIOR" };
        println!(
            "{:<6} {:08X}     {:04X}     {:<5} {:<5} {}",
            i,
            TRACE_BASE + i * 2,
            val,
            tile,
            pal,
            typ
        );
    }

    // Analyze: which PCs wrote correct vs wrong values
    println!("\n=== Analysis: PCs writing to screen entries ===");
    let mut pc_stats: BTreeMap<u32, (usize, usize)> = BTreeMap::new();
    for entry_idx in 0..64u32 {
        let addr = TRACE_BASE + entry_idx * 2;
        let voff = vram_base + (entry_idx * 2) as usize;
        let final_val = u16::from_le_bytes([vram[voff], vram[voff + 1]]);
        let is_border = entry_idx == 0 || entry_idx == 31 || entry_idx == 32 || entry_idx == 63;

        if let Some(writes) = writes_by_addr.get(&addr) {
            if let Some((val, pc)) = writes.last() {
                let is_correct = if is_border { true } else { *val == final_val };
                let stats = pc_stats.entry(*pc).or_insert((0, 0));
                if is_correct {
                    stats.0 += 1;
                } else {
                    stats.1 += 1;
                }
            }
        }
    }

    println!("{:<12} {:<10} {:<10} {}", "PC", "Correct", "Wrong", "Notes");
    println!("{}", "-".repeat(50));
    for (pc, (correct, wrong)) in &pc_stats {
        let notes = if *wrong > 0 {
            "SUSPECT - wrote wrong values"
        } else {
            ""
        };
        println!("{:<08X}   {:<10} {:<10} {}", pc, correct, wrong, notes);
    }

    // Show detailed diff: expected vs actual for interior entries
    println!("\n=== Interior entries with wrong values ===");
    println!(
        "{:<6} {:<14} {:<10} {:<10} {:<12}",
        "Idx", "Address", "Written", "Final", "WritePC"
    );
    println!("{}", "-".repeat(60));
    for entry_idx in 0..64u32 {
        let is_border = entry_idx == 0 || entry_idx == 31 || entry_idx == 32 || entry_idx == 63;
        if is_border {
            continue;
        }
        let addr = TRACE_BASE + entry_idx * 2;
        let voff = vram_base + (entry_idx * 2) as usize;
        let final_val = u16::from_le_bytes([vram[voff], vram[voff + 1]]);
        let final_pal = (final_val >> 12) & 0xF;

        if let Some(writes) = writes_by_addr.get(&addr) {
            if let Some((val, pc)) = writes.last() {
                if *val != final_val {
                    println!(
                        "{:<6} {:08X}     {:04X}(p{})  {:04X}(p{})  {:08X}",
                        entry_idx,
                        addr,
                        val,
                        (val >> 12) & 0xF,
                        final_val,
                        final_pal,
                        pc
                    );
                }
            }
        }
    }
}

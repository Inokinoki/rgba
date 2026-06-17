use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let mut gba = Gba::new();
    gba.load_rom_path(rom_path).unwrap();

    gba.mem.dma_log_enabled = true;

    for _ in 0..100 {
        gba.run_frame();
    }

    let dma_log = &gba.mem.dma_log;
    println!(
        "=== DMA transfers after 100 frames: {} total ===",
        dma_log.len()
    );

    let mut vblank_dma = 0;
    let mut immediate_dma = 0;
    let mut hblank_dma = 0;
    let mut special_dma = 0;
    let mut vram_writes = 0;

    for &(ch, src, dst, count, size) in dma_log {
        let is_vram_dst = (dst >= 0x06000000 && dst < 0x06018000);
        let is_vram_src = (src >= 0x06000000 && src < 0x06018000);

        if ch == 0 {
            vblank_dma += 1;
        } else if ch == 1 {
            immediate_dma += 1;
        } else if ch == 2 {
            hblank_dma += 1;
        } else {
            special_dma += 1;
        }

        if is_vram_dst {
            vram_writes += 1;
        }
    }

    println!("DMA0 (VBlank?): {}", vblank_dma);
    println!("DMA1 (Immediate?): {}", immediate_dma);
    println!("DMA2 (HBlank?): {}", hblank_dma);
    println!("DMA3 (Special?): {}", special_dma);
    println!("DMA writes to VRAM: {}", vram_writes);

    println!("\n=== First 30 DMA transfers ===");
    for (i, &(ch, src, dst, count, size)) in dma_log.iter().take(30).enumerate() {
        let is_vram_dst = (dst >= 0x06000000 && dst < 0x06018000);
        let is_vram_src = (src >= 0x06000000 && src < 0x06018000);
        println!(
            "  [{}] DMA{}: src={:#010X} dst={:#010X} count={} size={}{}{}",
            i,
            ch,
            src,
            dst,
            count,
            size,
            if is_vram_src { " [VRAM_SRC]" } else { "" },
            if is_vram_dst { " [VRAM_DST]" } else { "" }
        );
    }

    println!("\n=== Last 30 DMA transfers ===");
    let start = dma_log.len().saturating_sub(30);
    for (i, &(ch, src, dst, count, size)) in dma_log.iter().enumerate().skip(start) {
        let is_vram_dst = (dst >= 0x06000000 && dst < 0x06018000);
        let is_vram_src = (src >= 0x06000000 && src < 0x06018000);
        println!(
            "  [{}] DMA{}: src={:#010X} dst={:#010X} count={} size={}{}{}",
            i,
            ch,
            src,
            dst,
            count,
            size,
            if is_vram_src { " [VRAM_SRC]" } else { "" },
            if is_vram_dst { " [VRAM_DST]" } else { "" }
        );
    }

    println!("\n=== DMA transfer summary by channel ===");
    for ch in 0..4 {
        let entries: Vec<_> = dma_log.iter().filter(|&&(c, _, _, _, _)| c == ch).collect();
        if entries.is_empty() {
            println!("  DMA{}: no transfers", ch);
            continue;
        }
        let vram_count = entries
            .iter()
            .filter(|e| e.2 >= 0x06000000 && e.2 < 0x06018000)
            .count();
        let total_bytes: u64 = entries.iter().map(|e| e.3 as u64 * e.4 as u64).sum();
        println!(
            "  DMA{}: {} transfers, {} to VRAM, {} total bytes",
            ch,
            entries.len(),
            vram_count,
            total_bytes
        );
    }

    println!("\n=== VRAM state after 100 frames ===");
    let vram = gba.mem.vram();
    let mut non_zero = 0;
    let mut non_ff = 0;
    for &b in vram.iter() {
        if b != 0 {
            non_zero += 1;
        }
        if b != 0xFF {
            non_ff += 1;
        }
    }
    println!("VRAM size: {} bytes", vram.len());
    println!(
        "Non-zero bytes: {}/{} ({:.1}%)",
        non_zero,
        vram.len(),
        non_zero as f64 / vram.len() as f64 * 100.0
    );
    println!(
        "Non-0xFF bytes: {}/{} ({:.1}%)",
        non_ff,
        vram.len(),
        non_ff as f64 / vram.len() as f64 * 100.0
    );

    println!("\n=== VRAM tile data at char_base=0 (first 256 bytes) ===");
    for row in 0..16 {
        let offset = row * 16;
        print!("  {:04X}: ", offset);
        for col in 0..16 {
            print!("{:02X} ", vram[offset + col]);
        }
        println!();
    }

    println!("\n=== BG screen entries at screen_base 0xC000 (BG0) ===");
    for row in 0..8 {
        print!("  row {}: ", row);
        for col in 0..16 {
            let offset = 0xC000 + (row * 32 + col) * 2;
            let entry = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
            print!("{:03X} ", entry & 0x3FF);
        }
        println!();
    }

    println!("\n=== DMA register state ===");
    for ch in 0..4 {
        let d = &gba.dma[ch];
        println!("  DMA{}: enabled={} active={} trigger={:?} src={:#010X} dst={:#010X} count={} repeat={}",
            ch, d.is_enabled(), d.is_active(), d.get_trigger(),
            d.get_src_addr(), d.get_dst_addr(), d.get_count(), d.is_repeat());
    }

    println!("\n=== PPU BG register state ===");
    let dc = gba.ppu().get_dispcnt();
    println!(
        "DISPCNT: {:#06X} (mode={} BG={} OBJ={})",
        dc,
        dc & 7,
        (dc >> 8) & 0xF,
        (dc >> 12) & 1
    );
    for bg in 0..4 {
        let bgcnt = gba.ppu().get_bgcnt(bg);
        let priority = bgcnt & 3;
        let char_base = ((bgcnt >> 2) & 3) * 0x4000;
        let screen_base = ((bgcnt >> 8) & 0x1F) * 0x800;
        let size = (bgcnt >> 14) & 3;
        println!(
            "  BG{}: BGCTL={:#06X} pri={} char_base={:#06X} screen_base={:#06X} size={}",
            bg, bgcnt, priority, char_base, screen_base, size
        );
    }

    println!("\n=== IE/IME/IF ===");
    println!("IE: {:?}", gba.mem.interrupt.ie);
    println!("IME: {}", gba.mem.interrupt.ime);
    println!("IF: {:?}", gba.mem.interrupt.if_raw);
}

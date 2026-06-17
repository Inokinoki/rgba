use rgba::Gba;

fn count_tiles(vram: &[u8], start: u32, end: u32) -> u32 {
    let mut count = 0u32;
    for t in start..end {
        let offset = t as usize * 32;
        if offset + 32 > vram.len() { break; }
        let mut has = false;
        for b in 0..32 { if vram[offset + b] != 0 { has = true; break; } }
        if has { count += 1; }
    }
    count
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];
    for _ in 0..240 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    // Copy memory VRAM to compare
    let mem_vram = gba.mem().vram().to_vec();
    let ppu_vram = gba.ppu().vram().to_vec();
    
    println!("Memory VRAM size: {}", mem_vram.len());
    println!("PPU VRAM size: {}", ppu_vram.len());
    
    for range in [(0, 512u32), (0, 1024u32), (0, 2048u32), (512, 1024u32)] {
        let mem_count = count_tiles(&mem_vram, range.0, range.1);
        let ppu_count = count_tiles(&ppu_vram, range.0, range.1);
        println!("Tiles {}-{}: mem={} ppu={}", range.0, range.1-1, mem_count, ppu_count);
    }
    
    // Sync and recheck
    gba.sync_ppu_full();
    let ppu_vram2 = gba.ppu().vram().to_vec();
    for range in [(0, 512u32), (0, 1024u32), (0, 2048u32)] {
        let ppu_count = count_tiles(&ppu_vram2, range.0, range.1);
        println!("After sync - Tiles {}-{}: ppu={}", range.0, range.1-1, ppu_count);
    }
    
    // Compare byte by byte
    let mut diff_count = 0u32;
    let mut first_diff = None;
    for i in 0..mem_vram.len().min(ppu_vram2.len()) {
        if mem_vram[i] != ppu_vram2[i] {
            diff_count += 1;
            if first_diff.is_none() {
                first_diff = Some((i, mem_vram[i], ppu_vram2[i]));
            }
        }
    }
    println!("Bytes different after sync: {}/{}", diff_count, mem_vram.len().min(ppu_vram2.len()));
    if let Some((off, m, p)) = first_diff {
        println!("First diff at offset {:#06X}: mem={:#04X} ppu={:#04X}", off, m, p);
    }
    
    // Check non-zero regions in memory VRAM
    println!("\nMemory VRAM non-zero byte distribution:");
    let region_size = 0x2000usize;
    for start in (0..mem_vram.len()).step_by(region_size) {
        let end = (start + region_size).min(mem_vram.len());
        let nonzero = mem_vram[start..end].iter().filter(|&&b| b != 0).count();
        println!("  {:#06X}-{:#06X}: {}/{} non-zero bytes", start, end-1, nonzero, end-start);
    }
}

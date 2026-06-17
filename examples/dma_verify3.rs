use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];
    gba.mem_mut().dma_log_enabled = true;
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

    let vram = gba.mem().vram();
    let rom = gba.mem().rom();

    // Check last 5 DMA transfers from ROM to VRAM tile area
    let dma_log = &gba.mem().dma_log;
    let vram_dmas: Vec<_> = dma_log
        .iter()
        .rev()
        .filter(|&&(ch, src, dst, cnt, sz)| dst >= 0x06000000 && dst < 0x06010000 && cnt > 10)
        .take(5)
        .collect();

    println!("=== Verify DMA transfers ===");
    for &&(ch, src, dst, cnt, sz) in &vram_dmas {
        let total_bytes = cnt as usize * sz as usize;
        let vram_off = (dst - 0x06000000) as usize;
        let rom_off = src.checked_sub(0x08000000).unwrap_or(0xFFFFFFFF) as usize;

        if rom_off < rom.len() && vram_off + total_bytes <= vram.len() {
            let mut match_count = 0;
            for i in 0..total_bytes {
                if rom_off + i < rom.len() {
                    let rom_byte = if sz == 4 {
                        // DMA word transfer: read 4 bytes at aligned address
                        let word_off = (i / 4) * 4;
                        rom[rom_off + word_off + (i % 4)]
                    } else {
                        // DMA halfword transfer
                        let half_off = (i / 2) * 2;
                        rom[rom_off + half_off + (i % 2)]
                    };
                    if vram[vram_off + i] == rom_byte {
                        match_count += 1;
                    }
                }
            }
            println!(
                "DMA{} src={:#X} dst={:#X} cnt={} sz={}: {}/{} match ({:.0}%)",
                ch,
                src,
                dst,
                cnt,
                sz,
                match_count,
                total_bytes,
                match_count as f64 / total_bytes as f64 * 100.0
            );
        } else {
            println!(
                "DMA{} src={:#X} dst={:#X}: out of range (rom_off={:#X})",
                ch, src, dst, rom_off
            );
        }
    }

    // Also check: does the screen block data in VRAM match what a
    // correct farm scene would look like?
    // Check BG3 screen block at 0xE000 - all entries should be tile 279 (grass)
    let bg3_map = 0xE000;
    let mut tile_counts: std::collections::HashMap<u16, u32> = std::collections::HashMap::new();
    for i in 0..1024 {
        let e = u16::from_le_bytes([vram[bg3_map + i * 2], vram[bg3_map + i * 2 + 1]]);
        *tile_counts.entry(e & 0x3FF).or_insert(0) += 1;
    }
    let mut sorted: Vec<_> = tile_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    println!("\n=== BG3 screen block (0xE000) tile distribution ===");
    for (tile, count) in sorted.iter().take(10) {
        println!("  Tile {}: {} entries", tile, count);
    }
}

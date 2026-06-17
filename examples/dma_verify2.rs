use rgba::Gba;
use rgba::KeyState;
fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];

    gba.mem_mut().dma_log_enabled = true;
    gba.mem_mut().swi_log_enabled = true;
    gba.mem_mut().cpu_set_log_enabled = true;

    for _ in 0..240 { gba.run_frame_parallel(&mut framebuffer); }
    gba.input_mut().press_key(KeyState::START);
    for _ in 0..60 { gba.run_frame_parallel(&mut framebuffer); }
    gba.input_mut().release_key(KeyState::START);
    for _ in 0..120 { gba.run_frame_parallel(&mut framebuffer); }
    for _ in 0..100 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..3 { gba.run_frame_parallel(&mut framebuffer); }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..12 { gba.run_frame_parallel(&mut framebuffer); }
    }

    // Check the last DMA to VRAM tile area (0x06000000-0x0000FFFF)
    let dma_log = &gba.mem().dma_log;
    let vram_tiled: Vec<_> = dma_log.iter().rev()
        .filter(|&&(ch, src, dst, cnt, sz)| {
            dst >= 0x06000000 && dst < 0x06010000 && cnt > 10
        })
        .take(5)
        .collect();

    println!("=== Recent large DMA to BG tile area ===");
    for &&(ch, src, dst, cnt, sz) in &vram_tiled {
        println!("  DMA{} src={:#X} dst={:#X} count={} size={}", ch, src, dst, cnt, sz);
        // Verify first 4 words
        let vram = gba.mem().vram();
        let rom = gba.mem().rom_data();
        let dst_offset = (dst - 0x06000000) as usize;
        let src_offset = (src - 0x08000000) as usize;
        if src_offset + 16 <= rom.len() && dst_offset + 16 <= vram.len() {
            let mut match_bytes = 0;
            let check_len = (cnt as usize * sz as usize).min(256);
            for i in 0..check_len {
                if src_offset + i < rom.len() && dst_offset + i < vram.len() {
                    if rom[src_offset + i] == vram[dst_offset + i] {
                        match_bytes += 1;
                    }
                }
            }
            println!("    First {} bytes: {} match ROM ({:.0}%)", check_len, match_bytes, match_bytes as f64 / check_len as f64 * 100.0);
        }
    }

    // Check the most recent CpuFastSet to VRAM
    let cpu_set_log = &gba.mem().cpu_set_log;
    let vram_csets: Vec<_> = cpu_set_log.iter().rev()
        .filter(|&&(src, dst, cnt)| {
            dst >= 0x06000000 && dst < 0x06010000
        })
        .take(5)
        .collect();

    println!("\n=== Recent CpuFastSet to BG tile area ===");
    for &(src, dst, cnt) in &vram_csets {
        let fill = (cnt >> 24) & 1 != 0;
        let count = cnt & 0x1FFFFF;
        println!("  src={:#X} dst={:#X} cnt={:#X} fill={} count={}", src, dst, cnt, fill, count);
        let vram = gba.mem().vram();
        let rom = gba.mem().rom_data();
        let dst_offset = (dst - 0x06000000) as usize;
        let src_offset = (src - 0x08000000) as usize;
        if src_offset + 16 <= rom.len() && dst_offset + 16 <= vram.len() && !fill {
            let mut match_bytes = 0;
            let check_len = (count as usize * 4).min(256);
            for i in 0..check_len {
                if src_offset + i < rom.len() && dst_offset + i < vram.len() {
                    if rom[src_offset + i] == vram[dst_offset + i] {
                        match_bytes += 1;
                    }
                }
            }
            println!("    First {} bytes: {} match ROM ({:.0}%)", check_len, match_bytes, match_bytes as f64 / check_len as f64 * 100.0);
        }
    }
}

use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    gba.mem_mut().dma_log_enabled = true;

    let mut framebuffer = vec![0u32; 240 * 160];

    for _ in 0..10 { gba.run_frame_parallel(&mut framebuffer); }

    let log = &gba.mem().dma_log;
    println!("DMA transfers in first 10 frames: {}", log.len());
    for (i, &(ch, src, dst, count, size)) in log.iter().enumerate() {
        if i >= 20 { break; }
        println!("  DMA{}: {:#010X} -> {:#010X}, count={} sz={}", ch, src, dst, count, size);
    }
}

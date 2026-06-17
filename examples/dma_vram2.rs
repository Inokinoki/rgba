use rgba::Dma;
use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem_mut().dma_log_enabled = true;
    gba.mem_mut().dma_log.clear();

    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let log = &gba.mem().dma_log;
    println!("DMA transfers: {}", log.len());

    let mut vram_dma = 0;
    for (ch, src, dst, count, size) in log {
        if *dst >= 0x0600_0000 && *dst < 0x0602_0000 {
            vram_dma += 1;
            let vram_off = dst - 0x0600_0000;
            let is_char = vram_off < 0x10000;
            println!(
                "  DMA{}: src={:#010X} dst=VRAM+{:#X} count={} size={} {}",
                ch,
                src,
                vram_off,
                count,
                size,
                if is_char { "[CHAR]" } else { "[SCREEN/OBJ]" }
            );
        }
    }
    println!("\nDMA to VRAM: {}", vram_dma);
}

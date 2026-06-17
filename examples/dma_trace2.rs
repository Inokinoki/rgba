use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    gba.mem.dma_log_enabled = true;

    for f in 0..201 {
        gba.mem.dma_log.clear();
        gba.run_frame_parallel(&mut fb);
        if f >= 195 {
            let n = gba.mem.dma_log.len();
            println!("Frame {}: {} DMA transfers", f, n);
            for &(ch, src, dst, cnt, ctrl) in &gba.mem.dma_log {
                let is_vram = dst >= 0x06000000 && dst < 0x06020000;
                if is_vram || n <= 20 {
                    println!(
                        "  DMA{}: 0x{:08X}->0x{:08X} cnt={} ctrl=0x{:08X}{}",
                        ch,
                        src,
                        dst,
                        cnt,
                        ctrl,
                        if is_vram { " [VRAM]" } else { "" }
                    );
                }
            }
        }
    }
}

use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.dma_log_enabled = true;

    for frame in 0..300u32 {
        let dma_before = gba.mem.dma_log.len();
        gba.run_frame_parallel(&mut fb);

        // Check for DMA to/from OBJ palette (0x05000200-0x05000400) or involving EWRAM 0x2008000
        for (ch, dst, src, cnt, ctrl) in &gba.mem.dma_log[dma_before..] {
            let involves_pal = (*src >= 0x05000000 && *src < 0x05000400)
                || (*dst >= 0x05000000 && *dst < 0x05000400);
            let involves_8k = (*src >= 0x02008000 && *src < 0x02009000)
                || (*dst >= 0x02008000 && *dst < 0x02009000);
            let involves_decomp = (*src >= 0x02000000 && *src < 0x02004000)
                || (*dst >= 0x02000000 && *dst < 0x02004000);

            if involves_pal || involves_8k || involves_decomp {
                println!(
                    "Frame {:4} DMA{} src={:08X} dst={:08X} cnt={:3} ctrl={:08X} {}",
                    frame,
                    ch,
                    src,
                    dst,
                    cnt,
                    ctrl,
                    if involves_pal {
                        "PAL"
                    } else if involves_8k {
                        "8K"
                    } else {
                        "DECOMP"
                    }
                );
            }
        }
    }
}

use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..600 {
        gba.run_frame_parallel(&mut fb);
        if frame % 30 == 0 {
            gba.sync_ppu_full();
            let vram = gba.mem.vram();

            let mut nonzero_0343 = 0;
            let mut nonzero_344_472 = 0;
            let mut nonzero_473_825 = 0;
            let mut nonzero_826_1023 = 0;

            for tid in 0..=343u32 {
                let off = tid as usize * 32;
                if vram[off..off + 32].iter().any(|&b| b != 0) {
                    nonzero_0343 += 1;
                }
            }
            for tid in 344..=472u32 {
                let off = tid as usize * 32;
                if vram[off..off + 32].iter().any(|&b| b != 0) {
                    nonzero_344_472 += 1;
                }
            }
            for tid in 473..=825u32 {
                let off = tid as usize * 32;
                if vram[off..off + 32].iter().any(|&b| b != 0) {
                    nonzero_473_825 += 1;
                }
            }
            for tid in 826..=1023u32 {
                let off = tid as usize * 32;
                if vram[off..off + 32].iter().any(|&b| b != 0) {
                    nonzero_826_1023 += 1;
                }
            }

            eprintln!(
                "Frame {:3}: 0-343:{} | 344-472:{} | 473-825:{} | 826-1023:{}",
                frame, nonzero_0343, nonzero_344_472, nonzero_473_825, nonzero_826_1023
            );
        }
    }

    // Final check: show all non-zero tile ranges
    gba.sync_ppu_full();
    let vram = gba.mem.vram();
    let mut ranges: Vec<(u32, u32)> = Vec::new();
    for tid in 0..1024u32 {
        let off = tid as usize * 32;
        let nonzero = vram[off..off + 32].iter().any(|&b| b != 0);
        if nonzero {
            if let Some(last) = ranges.last_mut() {
                if tid == last.1 + 1 {
                    last.1 = tid;
                } else {
                    ranges.push((tid, tid));
                }
            } else {
                ranges.push((tid, tid));
            }
        }
    }
    eprintln!("\nFinal non-zero tile ranges (frame 600):");
    for (start, end) in &ranges {
        eprintln!("  Tiles {}-{} ({} tiles)", start, end, end - start + 1);
    }
}

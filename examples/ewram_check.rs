use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for _ in 0..200 {
        for _ in 0..228 {
            gba.run_scanline();
        }
    }

    let wram = gba.mem().wram();
    println!("=== EWRAM nonzero regions (>= 128 bytes) ===");
    let mut region_start: Option<usize> = None;
    for i in (0..wram.len()).step_by(4) {
        let nonzero = wram[i..i + 4.min(wram.len() - i)].iter().any(|&b| b != 0);
        if nonzero && region_start.is_none() {
            region_start = Some(i);
        } else if !nonzero && region_start.is_some() {
            let start = region_start.unwrap();
            let size = i - start;
            if size >= 128 {
                println!(
                    "  {:08X}-{:08X} ({} bytes, ~{} tiles)",
                    0x02000000 + start,
                    0x02000000 + i,
                    size,
                    size / 32
                );
            }
            region_start = None;
        }
    }
    if let Some(start) = region_start {
        let size = wram.len() - start;
        if size >= 128 {
            println!(
                "  {:08X}-{:08X} ({} bytes, ~{} tiles)",
                0x02000000 + start,
                0x02000000 + wram.len(),
                size,
                size / 32
            );
        }
    }

    let iwram = gba.mem().iwram();
    println!("\n=== IWRAM nonzero regions (>= 128 bytes) ===");
    let mut region_start: Option<usize> = None;
    for i in (0..iwram.len()).step_by(4) {
        let nonzero = iwram[i..i + 4.min(iwram.len() - i)].iter().any(|&b| b != 0);
        if nonzero && region_start.is_none() {
            region_start = Some(i);
        } else if !nonzero && region_start.is_some() {
            let start = region_start.unwrap();
            let size = i - start;
            if size >= 128 {
                println!(
                    "  {:08X}-{:08X} ({} bytes)",
                    0x03000000 + start,
                    0x03000000 + i,
                    size
                );
            }
            region_start = None;
        }
    }
    if let Some(start) = region_start {
        let size = iwram.len() - start;
        if size >= 128 {
            println!(
                "  {:08X}-{:08X} ({} bytes)",
                0x03000000 + start,
                0x03000000 + iwram.len(),
                size
            );
        }
    }
}

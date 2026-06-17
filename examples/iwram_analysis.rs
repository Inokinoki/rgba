use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    for frame in 0..195u32 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    let iwram = gba.mem().iwram();
    let vram = gba.mem().vram();

    println!("=== IWRAM analysis ===");
    let mut nonzero_regions: Vec<(u32, u32, u32)> = Vec::new();
    let mut region_start = None;
    let mut region_count = 0u32;
    for i in 0..iwram.len() {
        if iwram[i] != 0 {
            if region_start.is_none() {
                region_start = Some(i as u32);
                region_count = 1;
            } else {
                region_count += 1;
            }
        } else {
            if region_start.is_some() && region_count > 16 {
                nonzero_regions.push((
                    region_start.unwrap(),
                    region_count,
                    0x03000000 + region_start.unwrap(),
                ));
                region_start = None;
                region_count = 0;
            } else {
                region_start = None;
                region_count = 0;
            }
        }
    }
    if region_start.is_some() && region_count > 16 {
        nonzero_regions.push((
            region_start.unwrap(),
            region_count,
            0x03000000 + region_start.unwrap(),
        ));
    }

    println!("Nonzero IWRAM regions (>16 bytes):");
    for (off, len, addr) in &nonzero_regions {
        let bytes: Vec<u8> = iwram[*off as usize..(*off as usize + 16.min(*len as usize))].to_vec();
        let hex: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
        println!(
            "  {:#010X}-{:?}: {} bytes: {}...",
            addr,
            off + len,
            len,
            hex.join("")
        );
    }

    println!("\n=== IWRAM at 0x03006DD0 (DMA source for map data) ===");
    let src1_off = 0x6DD0;
    if src1_off + 64 <= iwram.len() {
        for i in 0..32u32 {
            let off = src1_off + (i as usize) * 2;
            let entry = u16::from_le_bytes([iwram[off], iwram[off + 1]]);
            let tile = entry & 0x3FF;
            let pal = (entry >> 12) & 0xF;
            if entry != 0 {
                println!("  [{:3}] tile={} pal={}", i, tile, pal);
            }
        }
    }

    println!("\n=== Check: Is tile data in IWRAM? ===");
    let iwram_tile_start = 0x4000;
    for tile in [0u32, 1, 10, 50, 100, 200, 300] {
        let off = iwram_tile_off(tile);
        if off + 32 <= iwram.len() {
            let mut nonzero = 0;
            for b in 0..32 {
                if iwram[off + b] != 0 {
                    nonzero += 1;
                }
            }
            if nonzero > 0 {
                print!("IWRAM tile {}: {} nonzero bytes", tile, nonzero);
                if tile < 114 {
                    let vram_off = tile as usize * 32;
                    let mut vram_nz = 0;
                    for b in 0..32 {
                        if vram[vram_off + b] != 0 {
                            vram_nz += 1;
                        }
                    }
                    print!(" (VRAM has {} nz)", vram_nz);
                }
                println!();
            }
        }
    }
}

fn iwram_tile_off(tile: u32) -> usize {
    (tile * 32) as usize
}

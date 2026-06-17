use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let vram = gba.mem.vram();

    let text_tiles = [
        473, 474, 475, 476, 477, 478, 479, 480, 481, 491, 492, 493, 508, 509, 536, 537, 611, 689,
        738,
    ];

    println!("=== Tile pixel data for text tiles ===");
    for &tile_idx in &text_tiles {
        let base = tile_idx * 32;
        let mut nonzero = 0;
        let mut pixels = Vec::new();
        for row in 0..8 {
            for byte_off in 0..4 {
                let byte = vram[base + row * 4 + byte_off];
                let lo = byte & 0xF;
                let hi = (byte >> 4) & 0xF;
                if lo != 0 {
                    nonzero += 1;
                }
                if hi != 0 {
                    nonzero += 1;
                }
                pixels.push(format!("{:X}{:X}", lo, hi));
            }
        }
        println!(
            "Tile {}: {} non-zero pixels  first={}",
            tile_idx,
            nonzero,
            pixels.iter().take(16).cloned().collect::<Vec<_>>().join("")
        );
    }

    let tile_3ff_base = 0x3FF * 32;
    let mut nonzero_3ff = 0;
    for i in 0..32 {
        if vram[tile_3ff_base + i] != 0 {
            nonzero_3ff += 1;
        }
    }
    println!("\nTile 0x3FF: {} non-zero bytes", nonzero_3ff);

    let palette = gba.mem.palette();
    println!("\nPalette bank 11:");
    for i in 0..16 {
        let off = 11 * 32 + i * 2;
        let c = u16::from_le_bytes([palette[off], palette[off + 1]]);
        print!(" [{:2}]={:04X}", i, c);
    }
    println!();
}

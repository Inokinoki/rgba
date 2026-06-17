use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for _ in 0..1000 {
        gba.run_frame();
    }

    // Compare Memory VRAM vs PPU VRAM (before sync)
    let mem_vram = gba.mem().vram();
    let ppu_vram = gba.ppu().vram();

    println!("Memory VRAM len: {}", mem_vram.len());
    println!("PPU VRAM len: {}", ppu_vram.len());

    // Check tile 394 in memory VRAM
    let tile_off = 394 * 32;
    println!("\n=== Memory VRAM Tile 394 ===");
    let mut has_data = false;
    for byte in 0..32 {
        if mem_vram[tile_off + byte] != 0 {
            has_data = true;
        }
    }
    println!("Has data: {}", has_data);
    if has_data {
        for row in 0..8 {
            let row_off = tile_off + row * 4;
            print!("Row {}: ", row);
            for byte in 0..4 {
                print!("{:02X} ", mem_vram[row_off + byte]);
            }
            println!();
        }
    }

    // Check PPU VRAM tile 394
    println!("\n=== PPU VRAM Tile 394 ===");
    let mut has_data2 = false;
    for byte in 0..32 {
        if ppu_vram[tile_off + byte] != 0 {
            has_data2 = true;
        }
    }
    println!("Has data: {}", has_data2);

    // Count differences between Memory and PPU VRAM
    let min_len = mem_vram.len().min(ppu_vram.len());
    let mut diff_count = 0usize;
    let mut first_diff = 0usize;
    for i in 0..min_len {
        if mem_vram[i] != ppu_vram[i] {
            diff_count += 1;
            if first_diff == 0 {
                first_diff = i;
            }
        }
    }
    println!("\nVRAM differences: {} / {}", diff_count, min_len);
    if first_diff > 0 {
        println!(
            "First diff at offset {:#06X}: mem={:#04X} ppu={:#04X}",
            first_diff, mem_vram[first_diff], ppu_vram[first_diff]
        );
    }

    // Check how many non-zero bytes in Memory VRAM vs PPU VRAM for BG area
    let mut mem_nz = 0;
    let mut ppu_nz = 0;
    for i in 0..0x10000usize {
        if i < mem_vram.len() && mem_vram[i] != 0 {
            mem_nz += 1;
        }
        if i < ppu_vram.len() && ppu_vram[i] != 0 {
            ppu_nz += 1;
        }
    }
    println!("\nBG VRAM non-zero: mem={}, ppu={}", mem_nz, ppu_nz);

    // Screen entries in Memory VRAM
    println!("\n=== Memory VRAM BG0 screen entries (at 0xC000) ===");
    for i in 0..32 {
        let off = 0xC000 + i * 2;
        let e = u16::from_le_bytes([mem_vram[off], mem_vram[off + 1]]);
        let t = e & 0x3FF;
        if t != 0x3FF {
            print!("{:4}", t);
        } else {
            print!("   .");
        }
    }
    println!();

    // Now sync and check PPU VRAM
    gba.sync_ppu_full();
    let ppu_vram2 = gba.ppu().vram();

    println!("\n=== After sync_ppu_full: PPU VRAM Tile 394 ===");
    let mut has_data3 = false;
    for byte in 0..32 {
        if ppu_vram2[tile_off + byte] != 0 {
            has_data3 = true;
        }
    }
    println!("Has data: {}", has_data3);

    let mut ppu_nz2 = 0;
    for i in 0..0x10000usize {
        if i < ppu_vram2.len() && ppu_vram2[i] != 0 {
            ppu_nz2 += 1;
        }
    }
    println!("PPU BG VRAM non-zero after sync: {}", ppu_nz2);
}

use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();

    for frame in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let log = &gba.mem().vram_write_log;

    let mut cb3_offsets = std::collections::BTreeSet::new();
    for (addr, _pc, _val) in log {
        let offset = (addr - 0x0600_0000) as usize;
        if offset >= 0xC000 && offset < 0x10000 {
            cb3_offsets.insert(offset);
        }
    }

    println!(
        "Char block 3 (0xC000-0xFFFF) unique addresses: {}",
        cb3_offsets.len()
    );
    if !cb3_offsets.is_empty() {
        println!(
            "Range: {:#X} - {:#X}",
            cb3_offsets.iter().next().unwrap(),
            cb3_offsets.iter().next_back().unwrap()
        );
    }

    let vram = gba.mem().vram();

    println!("\n=== Tile data check for BG0 tiles at char_base 0 ===");
    let bg0_tiles = [394, 403, 412, 420, 473, 482, 491, 499];
    for &t in &bg0_tiles {
        let off = t * 32;
        let mut nonzero = 0;
        for b in 0..32 {
            if vram[off + b] != 0 {
                nonzero += 1;
            }
        }
        print!("  Tile {}: {} nonzero ", t, nonzero);
        if nonzero > 0 {
            print!("data: ");
            for b in 0..8 {
                print!("{:02X}", vram[off + b]);
            }
            println!("...");
        } else {
            println!();
        }
    }

    println!("\n=== Screen block at 0xC000 (BG0 map) first 4 rows ===");
    for ty in 0..4 {
        for tx in 0..32 {
            let off = 0xC000 + (ty * 32 + tx) * 2;
            let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
            let tile = entry & 0x3FF;
            print!("{:4} ", tile);
        }
        println!();
    }

    println!("\n=== Check if BG0 uses char_base 3 instead ===");
    let io = gba.mem().io();
    for bg in 0..4 {
        let bgcnt = u16::from_le_bytes([io[8 + bg * 2], io[9 + bg * 2]]);
        let char_base = ((bgcnt >> 2) & 3) * 0x4000;
        let screen_base = ((bgcnt >> 8) & 0x1F) * 0x800;
        println!(
            "BG{}: bgcnt={:#06X} char_base={:#X} screen_base={:#X}",
            bg, bgcnt, char_base, screen_base
        );
    }
}

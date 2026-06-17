use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for _ in 0..1000 {
        gba.run_frame();
    }

    let vram = gba.mem().vram();

    // Find which tiles have data in BG VRAM (0x0000-0xFFFF)
    println!("=== Non-empty tiles in BG VRAM ===");
    let mut nonempty_tiles = vec![];
    for tile in 0..1024u32 {
        let off = tile * 32;
        let mut has_data = false;
        for byte in 0..32u32 {
            if vram[(off + byte) as usize] != 0 {
                has_data = true;
                break;
            }
        }
        if has_data {
            nonempty_tiles.push(tile);
        }
    }
    println!("Non-empty tiles: {}/1024", nonempty_tiles.len());

    // Show first 20 non-empty tiles with their content
    for &tile in nonempty_tiles.iter().take(20) {
        let off = (tile * 32) as usize;
        print!("Tile {}: ", tile);
        for byte in 0..8 {
            print!("{:02X}", vram[off + byte]);
        }
        print!("...");
        for byte in 24..32 {
            print!("{:02X}", vram[off + byte]);
        }
        println!();
    }

    // Show which tiles are referenced by each BG's tilemap
    let io = gba.mem().io();
    for bg in 0..4u16 {
        let bgcnt_off = (0x08 + bg * 2) as usize;
        let bgcnt = u16::from_le_bytes([io[bgcnt_off], io[bgcnt_off + 1]]);
        let screen_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
        let size = (bgcnt >> 14) & 3;
        let enabled = {
            let dispcnt = u16::from_le_bytes([io[0], io[1]]);
            (dispcnt >> (8 + bg)) & 1
        };

        if enabled == 0 {
            continue;
        }

        let entry_count = match size {
            0 => 32 * 32,
            1 => 64 * 32,
            2 => 32 * 64,
            3 => 64 * 64,
            _ => 32 * 32,
        };

        let mut referenced_tiles = std::collections::HashSet::new();
        for i in 0..entry_count {
            let off = screen_base + i * 2;
            if off + 1 < vram.len() {
                let e = u16::from_le_bytes([vram[off], vram[off + 1]]);
                let t = e & 0x3FF;
                if t != 0x3FF {
                    referenced_tiles.insert(t as u32);
                }
            }
        }

        let mut referenced: Vec<u32> = referenced_tiles.into_iter().collect();
        referenced.sort();

        // Check which referenced tiles have data
        let mut empty_ref = 0;
        for &t in &referenced {
            let off = (t * 32) as usize;
            let mut has_data = false;
            for byte in 0..32 {
                if vram[off + byte] != 0 {
                    has_data = true;
                    break;
                }
            }
            if !has_data {
                empty_ref += 1;
            }
        }

        println!(
            "\nBG{}: {} unique tiles referenced, {} empty (no pixel data)",
            bg,
            referenced.len(),
            empty_ref
        );
        print!("  Referenced: ");
        for &t in referenced.iter().take(20) {
            let off = (t * 32) as usize;
            let mut has_data = false;
            for byte in 0..32 {
                if vram[off + byte] != 0 {
                    has_data = true;
                    break;
                }
            }
            print!("{}{} ", t, if has_data { "" } else { "*" });
        }
        println!();
    }

    // Check non-zero byte distribution in VRAM
    println!("\n=== VRAM non-zero byte distribution ===");
    let regions = [
        ("Tile 0-255 (0x0000-0x1FFF)", 0x0000usize, 0x2000),
        ("Tile 256-511 (0x2000-0x3FFF)", 0x2000, 0x2000),
        ("Tile 512-767 (0x4000-0x5FFF)", 0x4000, 0x2000),
        ("Tile 768-1023 (0x6000-0x7FFF)", 0x6000, 0x2000),
        ("Screen 0xC000-0xDFFF", 0xC000, 0x2000),
        ("Screen 0xE000-0xFFFF", 0xE000, 0x2000),
        ("OBJ VRAM (0x10000-0x17FFF)", 0x10000, 0x8000),
    ];
    for (name, start, size) in &regions {
        let mut nz = 0;
        for i in *start..(*start + *size) {
            if i < vram.len() && vram[i] != 0 {
                nz += 1;
            }
        }
        println!("  {}: {}", name, nz);
    }
}

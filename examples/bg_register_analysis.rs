use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    for frame in 0..195u32 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    let vram = gba.mem().vram();
    let io = gba.mem().io();

    let dispcnt = u16::from_le_bytes([io[0], io[1]]);
    println!("DISPCNT: {:#06X}", dispcnt);
    println!("  Mode: {}", dispcnt & 7);
    println!("  BG0: {}", (dispcnt >> 8) & 1);
    println!("  BG1: {}", (dispcnt >> 9) & 1);
    println!("  BG2: {}", (dispcnt >> 10) & 1);
    println!("  BG3: {}", (dispcnt >> 11) & 1);
    println!("  OBJ: {}", (dispcnt >> 12) & 1);

    for bg in 0..4u32 {
        let bgcnt_off = 0x08 + (bg as usize) * 2;
        let bgcnt = u16::from_le_bytes([io[bgcnt_off], io[bgcnt_off + 1]]);
        let priority = bgcnt & 3;
        let char_base = ((bgcnt >> 2) & 3) as u32 * 0x4000;
        let screen_base = ((bgcnt >> 8) & 0x1F) as u32 * 0x800;
        let size = (bgcnt >> 14) & 3;
        let mosaic = (bgcnt >> 6) & 1;
        let color_mode = (bgcnt >> 7) & 1;
        println!(
            "BG{}CNT: {:#06X} priority={} char_base={:#06X} screen_base={:#06X} size={} mosaic={} {}bit",
            bg, bgcnt, priority, char_base, screen_base, size, mosaic,
            if color_mode == 1 { "256" } else { "16" }
        );
    }

    println!("\n=== VRAM Tile analysis per char_base block ===");
    for block in 0..4u32 {
        let base = block * 0x4000;
        let mut nonzero_tiles = 0;
        let mut last_nonzero = 0u32;
        let max_tiles = if block < 3 { 512 } else { 256 };
        for tile in 0..max_tiles {
            let off = base + tile * 32;
            let mut has_data = false;
            for b in 0..32 {
                if vram[off as usize + b as usize] != 0 {
                    has_data = true;
                    break;
                }
            }
            if has_data {
                nonzero_tiles += 1;
                last_nonzero = tile;
            }
        }
        println!(
            "  Block {} (base {:#06X}): {} nonzero tiles, last={}",
            block, base, nonzero_tiles, last_nonzero
        );
    }

    println!("\n=== Tile data at each char_base ===");
    for block in 0..4u32 {
        let base = block * 0x4000;
        print!("  Block {}:", block);
        let mut found_any = false;
        for tile in [0u32, 1, 10, 50, 100, 200] {
            if block < 3 || tile < 256 {
                let off = base + tile * 32;
                let mut nonzero = 0;
                for b in 0..32 {
                    if vram[off as usize + b as usize] != 0 {
                        nonzero += 1;
                    }
                }
                if nonzero > 0 {
                    found_any = true;
                    print!(" t{}({}nz)", tile, nonzero);
                }
            }
        }
        if !found_any {
            print!(" (empty)");
        }
        println!();
    }

    println!("\n=== Map data analysis per screen_base ===");
    for block in 0..32u32 {
        let base = block * 0x800;
        if base >= 0x18000 {
            break;
        }
        let mut nonzero = 0;
        let mut max_tile_ref = 0u32;
        for i in 0..(0x800 / 2) {
            let off = base + i * 2;
            if off as usize + 2 <= vram.len() {
                let entry = u16::from_le_bytes([vram[off as usize], vram[off as usize + 1]]);
                let tile_num = (entry & 0x3FF) as u32;
                if entry != 0 {
                    nonzero += 1;
                }
                if tile_num > max_tile_ref {
                    max_tile_ref = tile_num;
                }
            }
        }
        if nonzero > 0 {
            let screen_addr = 0x06000000 + base;
            println!(
                "  Screen block {} (base {:#06X}, addr {:#010X}): {} nonzero entries, max tile ref={}",
                block, base, screen_addr, nonzero, max_tile_ref
            );
        }
    }
}

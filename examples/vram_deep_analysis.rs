use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let rom = gba.mem().rom().to_vec();
    let mut framebuffer = vec![0u32; 240 * 160];

    gba.mem_mut().vram_log_enabled = true;

    for frame in 0..195u32 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    let log = &gba.mem().vram_write_log;
    let vram = gba.mem().vram().to_vec();

    println!("=== Tile data in VRAM vs all-zero analysis ===");
    println!("Checking if tiles 114-1023 are truly zero:");

    let mut first_nonzero_after_113 = None;
    for tile in 114..1024u32 {
        let base = tile as usize * 32;
        let mut nonzero = 0;
        for b in 0..32 {
            if vram[base + b] != 0 {
                nonzero += 1;
            }
        }
        if nonzero > 0 {
            first_nonzero_after_113 = Some(tile);
            break;
        }
    }
    match first_nonzero_after_113 {
        Some(t) => println!("First nonzero tile after 113: tile {}", t),
        None => println!("Tiles 114-1023 are all zero (confirmed)"),
    }

    println!("\n=== Literal pool analysis ===");
    let literal_pool_start = 0x080D0CB0;
    for i in 0..20u32 {
        let rom_off = (literal_pool_start - 0x08000000 + i * 4) as usize;
        if rom_off + 4 <= rom.len() {
            let val = u32::from_le_bytes([
                rom[rom_off],
                rom[rom_off + 1],
                rom[rom_off + 2],
                rom[rom_off + 3],
            ]);
            let pc = literal_pool_start + i * 4;
            if val >= 0x06000000 && val < 0x06018000 {
                let vram_off = val - 0x06000000;
                let tile = vram_off / 32;
                let byte = vram_off % 32;
                println!(
                    "{:08X}: {:08X} (VRAM+{:#X} tile {} byte {})",
                    pc, val, vram_off, tile, byte
                );
            } else if val >= 0x08000000 && val < 0x0A000000 {
                println!("{:08X}: {:08X} (ROM+{:#X})", pc, val, val - 0x08000000);
            } else if val >= 0x02000000 && val < 0x03000000 {
                println!("{:08X}: {:08X} (EWRAM+{:#X})", pc, val, val - 0x02000000);
            } else if val >= 0x03000000 && val < 0x04000000 {
                println!("{:08X}: {:08X} (IWRAM+{:#X})", pc, val, val - 0x03000000);
            } else {
                println!("{:08X}: {:08X} ({})", pc, val, val);
            }
        }
    }

    println!("\n=== EWRAM tile-related data ===");
    let ewram = gba.mem().wram();
    let mut nonzero_ewram = 0;
    for b in ewram.iter() {
        if *b != 0 {
            nonzero_ewram += 1;
        }
    }
    println!("EWRAM nonzero bytes: {}/{}", nonzero_ewram, ewram.len());

    let mut framebuffer2 = vec![0u32; 240 * 160];
    for frame in 195..300u32 {
        gba.run_frame_parallel(&mut framebuffer2);
    }

    let vram2 = gba.mem().vram();
    let mut new_tiles = 0;
    for tile in 0..1024u32 {
        let base = tile as usize * 32;
        let mut was_zero = true;
        let mut is_nonzero = false;
        for b in 0..32 {
            if vram[base + b] != 0 {
                was_zero = false;
            }
            if vram2[base + b] != 0 {
                is_nonzero = true;
            }
        }
        if was_zero && is_nonzero {
            new_tiles += 1;
            if new_tiles <= 5 {
                println!("NEW tile {} loaded after frame 195!", tile);
            }
        }
    }
    println!("New tiles loaded after frame 195: {}", new_tiles);

    let dispcnt_io = gba.mem().io();
    let dispcnt = u16::from_le_bytes([dispcnt_io[0], dispcnt_io[1]]);
    println!("\nDISPCNT at frame 300: {:#06X}", dispcnt);
    for bg in 0..4u32 {
        let off = 0x08 + bg as usize * 2;
        let bgcnt = u16::from_le_bytes([dispcnt_io[off], dispcnt_io[off + 1]]);
        if bgcnt != 0 {
            let char_base = ((bgcnt >> 2) & 3) as u32 * 0x4000;
            let screen_base = ((bgcnt >> 8) & 0x1F) as u32 * 0x800;
            let enabled = (dispcnt >> (8 + bg)) & 1;
            println!(
                "BG{}CNT: {:#06X} char={:#X} screen={:#X} enabled={}",
                bg, bgcnt, char_base, screen_base, enabled
            );
        }
    }
}

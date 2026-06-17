use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for _ in 0..1000 {
        gba.run_frame();
    }

    gba.sync_ppu_full();
    gba.sync_ppu();

    let vram = gba.ppu().vram();
    let pal = gba.mem().palette();

    // Tile 394 data
    let tile_off = 394 * 32;
    println!("=== Tile 394 (used in BG0) ===");
    for row in 0..8 {
        let row_off = tile_off + row * 4;
        print!("Row {}: ", row);
        for byte in 0..4 {
            let b = vram[row_off + byte];
            let lo = b & 0xF;
            let hi = (b >> 4) & 0xF;
            print!("{:X}{:X}", lo, hi);
        }
        println!();
    }

    // Render tile 394 as ASCII
    println!("\n=== Tile 394 visual (4bpp, pal_bank=0) ===");
    for row in 0..8 {
        let row_off = tile_off + row * 4;
        for byte in 0..4 {
            let b = vram[row_off + byte];
            let lo = b & 0xF;
            let hi = (b >> 4) & 0xF;
            if lo == 0 {
                print!(" ");
            } else {
                print!("{}", (b'0' + lo as u8) as char);
            }
            if hi == 0 {
                print!(" ");
            } else {
                print!("{}", (b'0' + hi as u8) as char);
            }
        }
        println!();
    }

    // Tile 0 data
    println!("\n=== Tile 0 ===");
    for row in 0..8 {
        let row_off = row * 4;
        let mut has_data = false;
        for byte in 0..4 {
            if vram[row_off + byte] != 0 {
                has_data = true;
                break;
            }
        }
        if has_data {
            print!("Row {}: ", row);
            for byte in 0..4 {
                let b = vram[row_off + byte];
                let lo = b & 0xF;
                let hi = (b >> 4) & 0xF;
                print!("{:X}{:X} ", lo, hi);
            }
            println!();
        }
    }

    // Check BG0 HOFS/VOFS - maybe scrolling is wrong
    let io = gba.mem().io();
    let bg0hofs = u16::from_le_bytes([io[0x10], io[0x11]]) & 0x1FF;
    let bg0vofs = u16::from_le_bytes([io[0x12], io[0x13]]) & 0x1FF;
    let bg1hofs = u16::from_le_bytes([io[0x14], io[0x15]]) & 0x1FF;
    let bg1vofs = u16::from_le_bytes([io[0x16], io[0x17]]) & 0x1FF;
    let bg2hofs = u16::from_le_bytes([io[0x18], io[0x19]]) & 0x1FF;
    let bg2vofs = u16::from_le_bytes([io[0x1A], io[0x1B]]) & 0x1FF;
    let bg3hofs = u16::from_le_bytes([io[0x1C], io[0x1D]]) & 0x1FF;
    let bg3vofs = u16::from_le_bytes([io[0x1E], io[0x1F]]) & 0x1FF;

    println!("\n=== BG Scroll offsets ===");
    println!("BG0: HOFS={} Vofs={}", bg0hofs, bg0vofs);
    println!("BG1: HOFS={} Vofs={}", bg1hofs, bg1vofs);
    println!("BG2: HOFS={} Vofs={}", bg2hofs, bg2vofs);
    println!("BG3: HOFS={} Vofs={}", bg3hofs, bg3vofs);

    // Now manually render a few pixels
    // BG0: priority=3, screen_base=0xC000, char_base=0, size=1 (64x32)
    // BG3: priority=0 (highest!), check BG3 settings
    let bg3cnt = u16::from_le_bytes([io[0x0E], io[0x0F]]);
    let bg3_char = ((bg3cnt >> 2) & 0xF) as u32 * 0x4000;
    let bg3_screen = ((bg3cnt >> 8) & 0x1F) as u32 * 0x800;
    let bg3_size = (bg3cnt >> 14) & 3;
    let bg3_pri = bg3cnt & 3;
    println!("\n=== BG3 (highest priority) ===");
    println!(
        "BGCNT={:#06X} pri={} char={:#06X} screen={:#06X} size={}",
        bg3cnt, bg3_pri, bg3_char, bg3_screen, bg3_size
    );

    // BG3 tilemap
    let bg3_screen_base = bg3_screen as usize;
    println!("BG3 first 32 screen entries:");
    for i in 0..32 {
        let off = bg3_screen_base + i * 2;
        let e = u16::from_le_bytes([vram[off], vram[off + 1]]);
        let t = e & 0x3FF;
        if t != 0x3FF {
            print!("{:4}", t);
        } else {
            print!("   .");
        }
    }
    println!();
}

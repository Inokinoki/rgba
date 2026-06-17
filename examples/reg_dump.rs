use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];
    for _ in 0..240 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    gba.sync_ppu_full();
    let io = gba.mem().io();
    let vram = gba.mem().vram();

    println!("=== Raw IO Register Bytes ===");
    println!("DISPCNT [0x00]: {:02X} {:02X}", io[0x00], io[0x01]);
    for bg in 0..4 {
        let off = 0x08 + bg * 2;
        println!(
            "BGCNT{} [0x{:02X}]: {:02X} {:02X} = {:#06X}",
            bg,
            off,
            io[off],
            io[off + 1],
            u16::from_le_bytes([io[off], io[off + 1]])
        );
    }
    for bg in 0..4 {
        let h_off = 0x10 + bg * 4;
        let v_off = h_off + 2;
        println!(
            "BG{}HOFS [0x{:02X}]: {:02X} {:02X} = {}  BG{}VOFS [0x{:02X}]: {:02X} {:02X} = {}",
            bg,
            h_off,
            io[h_off],
            io[h_off + 1],
            u16::from_le_bytes([io[h_off], io[h_off + 1]]) & 0x1FF,
            bg,
            v_off,
            io[v_off],
            io[v_off + 1],
            u16::from_le_bytes([io[v_off], io[v_off + 1]]) & 0x1FF
        );
    }

    println!("\n=== PPU BGCNT (via getter) ===");
    for bg in 0..4 {
        let bgcnt = gba.ppu().get_bgcnt(bg);
        let priority = bgcnt & 0x3;
        let char_base = (bgcnt >> 2) & 0x3;
        let mosaic = (bgcnt >> 6) & 1;
        let color_mode = (bgcnt >> 7) & 1;
        let screen_base = (bgcnt >> 8) & 0x1F;
        let overflow = (bgcnt >> 13) & 1;
        let size = (bgcnt >> 14) & 0x3;
        println!("BG{}: cnt={:#06X} pri={} char_base={} (tile={:#X}) screen_base={} (map={:#X}) mosaic={} {}bpp size={}",
            bg, bgcnt, priority, char_base, char_base as u16 * 0x4000,
            screen_base, screen_base as u16 * 0x800,
            mosaic, if color_mode != 0 { "8" } else { "4" }, size);
    }

    println!("\n=== VRAM Tile Data Distribution ===");
    for region in 0..4 {
        let base = region * 0x4000;
        let end = base + 0x4000;
        if end > vram.len() {
            break;
        }
        let nonzero = vram[base..end].iter().filter(|&&b| b != 0).count();
        let mut first_nonzero = None;
        let mut last_nonzero = None;
        for i in 0..0x4000 {
            if base + i < vram.len() && vram[base + i] != 0 {
                if first_nonzero.is_none() {
                    first_nonzero = Some(i);
                }
                last_nonzero = Some(i);
            }
        }
        println!(
            "  {:#06X}-{:#06X}: {} nonzero bytes, range {:#X}-{:#X}",
            base,
            end,
            nonzero,
            first_nonzero.unwrap_or(0),
            last_nonzero.unwrap_or(0)
        );
    }

    println!("\n=== First 16 non-zero tiles at base 0x0000 ===");
    let mut count = 0;
    for tile in 0..1024u32 {
        let off = tile as usize * 32;
        if off + 32 > vram.len() {
            break;
        }
        if (0..32).any(|i| vram[off + i] != 0) {
            print!("  tile {}: ", tile);
            for i in 0..8 {
                print!("{:02X}", vram[off + i]);
            }
            print!("...");
            println!();
            count += 1;
            if count >= 16 {
                break;
            }
        }
    }

    println!("\n=== BG0 Map Data at 0xC000 (first 64 bytes raw) ===");
    for row in 0..4 {
        print!("  ");
        for col in 0..16 {
            let off = 0xC000 + (row * 32 + col) * 2;
            let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
            print!("{:04X} ", entry);
        }
        println!();
    }
}

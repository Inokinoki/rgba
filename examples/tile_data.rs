use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().press_key(KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(KeyState::START);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut fb);
    }
    for _ in 0..40 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..5 {
            gba.run_frame_parallel(&mut fb);
        }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..55 {
            gba.run_frame_parallel(&mut fb);
        }
    }

    let vram = gba.ppu().vram();
    let pal = gba.mem().palette();

    // Show tile data for tiles 0-10
    println!("=== Tiles 0-10 (at tile_base=0x0000) ===");
    for tile_num in 0u16..=10 {
        let off = tile_num as usize * 32;
        let mut has_data = false;
        for i in 0..32 {
            if vram[off + i] != 0 {
                has_data = true;
                break;
            }
        }
        if !has_data {
            println!("Tile {}: all zeros", tile_num);
            continue;
        }
        println!("Tile {}:", tile_num);
        for row in 0..8 {
            let b0 = vram[off + row * 4];
            let b1 = vram[off + row * 4 + 1];
            let b2 = vram[off + row * 4 + 2];
            let b3 = vram[off + row * 4 + 3];
            print!("  ");
            for b in [b0, b1, b2, b3] {
                print!("{:X}{:X} ", b & 0xF, (b >> 4) & 0xF);
            }
            // Show palette colors for this row
            print!("  -> ");
            for b in [b0, b1, b2, b3] {
                let lo = b & 0xF;
                let hi = (b >> 4) & 0xF;
                if lo != 0 {
                    let c = u16::from_le_bytes([pal[lo as usize * 2], pal[lo as usize * 2 + 1]]);
                    let r = (c & 0x1F) as u32 * 255 / 31;
                    let g = ((c >> 5) & 0x1F) as u32 * 255 / 31;
                    let b = ((c >> 10) & 0x1F) as u32 * 255 / 31;
                    print!("p0[{}]=RGB({},{},{}) ", lo, r, g, b);
                }
                if hi != 0 {
                    let c = u16::from_le_bytes([pal[hi as usize * 2], pal[hi as usize * 2 + 1]]);
                    let r = (c & 0x1F) as u32 * 255 / 31;
                    let g = ((c >> 5) & 0x1F) as u32 * 255 / 31;
                    let b = ((c >> 10) & 0x1F) as u32 * 255 / 31;
                    print!("p0[{}]=RGB({},{},{}) ", hi, r, g, b);
                }
            }
            println!();
        }
    }

    // Also show tile 279 (grass tile from BG3)
    for tile_num in [279u16, 277, 283] {
        let off = tile_num as usize * 32;
        let pal_off = 4 * 16 * 2; // palette 4
        println!("\nTile {} (palette 4):", tile_num);
        for row in 0..8 {
            let b0 = vram[off + row * 4];
            let b1 = vram[off + row * 4 + 1];
            let b2 = vram[off + row * 4 + 2];
            let b3 = vram[off + row * 4 + 3];
            print!("  ");
            for b in [b0, b1, b2, b3] {
                print!("{:X}{:X} ", b & 0xF, (b >> 4) & 0xF);
            }
            print!("  -> ");
            for b in [b0, b1, b2, b3] {
                let lo = b & 0xF;
                let hi = (b >> 4) & 0xF;
                for idx in [lo, hi] {
                    if idx != 0 {
                        let pi = pal_off + idx as usize * 2;
                        let c = u16::from_le_bytes([pal[pi], pal[pi + 1]]);
                        let r = (c & 0x1F) as u32 * 255 / 31;
                        let g = ((c >> 5) & 0x1F) as u32 * 255 / 31;
                        let b = ((c >> 10) & 0x1F) as u32 * 255 / 31;
                        print!("[{}]=RGB({},{},{}) ", idx, r, g, b);
                    }
                }
            }
            println!();
        }
    }
}

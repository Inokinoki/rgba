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
    gba.input.press_key(KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input.release_key(KeyState::START);
    for _ in 0..180 {
        gba.run_frame_parallel(&mut fb);
    }
    for _ in 0..80 {
        gba.input.press_key(KeyState::A);
        for _ in 0..4 {
            gba.run_frame_parallel(&mut fb);
        }
        gba.input.release_key(KeyState::A);
        for _ in 0..16 {
            gba.run_frame_parallel(&mut fb);
        }
    }

    gba.sync_ppu_full();
    let ppu = gba.ppu();
    let vram = ppu.vram();

    let test_tiles = [
        0, 1, 2, 15, 16, 70, 103, 125, 140, 206, 212, 228, 260, 262, 284, 324,
    ];

    for &tile in &test_tiles {
        let offset = tile as usize * 32;
        print!("Tile {:3} ({:#06X}): ", tile, offset);
        for b in 0..32 {
            print!("{:02X}", vram[offset + b]);
        }
        println!();

        let palette = gba.mem().palette();
        print!("  Colors: ");
        for y in 0..8 {
            for x in 0..8 {
                let byte = vram[offset + y * 4 + x / 2];
                let nibble = if x % 2 == 0 { byte & 0x0F } else { byte >> 4 };
                let pal_off = nibble as usize * 2;
                if nibble != 0 && pal_off + 1 < palette.len() {
                    let c = u16::from_le_bytes([palette[pal_off], palette[pal_off + 1]]);
                    let r = c & 0x1F;
                    let g = (c >> 5) & 0x1F;
                    let b = (c >> 10) & 0x1F;
                    print!("({},{},{})", r, g, b);
                }
            }
        }
        println!();
    }
}

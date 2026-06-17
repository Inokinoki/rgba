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

    gba.mem_mut().vram_log_enabled = true;

    for round in 0..200 {
        gba.input.press_key(KeyState::A);
        for _ in 0..4 {
            gba.run_frame_parallel(&mut fb);
        }
        gba.input.release_key(KeyState::A);
        for _ in 0..16 {
            gba.run_frame_parallel(&mut fb);
        }

        let vram = gba.mem().vram();
        let mut nonzero = 0;
        let mut last_tile = 0u32;
        for tile in 0..1024u32 {
            let base = tile as usize * 32;
            let mut has_data = false;
            for b in 0..32 {
                if vram[base + b] != 0 {
                    has_data = true;
                    break;
                }
            }
            if has_data {
                nonzero += 1;
                last_tile = tile;
            }
        }

        if nonzero > 120 || round % 50 == 0 {
            println!(
                "Round {}: {} nonzero tiles, last={}",
                round, nonzero, last_tile
            );
        }
    }

    gba.mem_mut().vram_log_enabled = false;

    let vram = gba.mem().vram();
    let mut nonzero = 0;
    for tile in 0..1024u32 {
        let base = tile as usize * 32;
        for b in 0..32 {
            if vram[base + b] != 0 {
                nonzero += 1;
                break;
            }
        }
    }
    println!("\nFinal: {} nonzero tiles", nonzero);

    let io = gba.mem().io();
    let dispcnt = u16::from_le_bytes([io[0], io[1]]);
    println!("DISPCNT: {:#06X}", dispcnt);
    for bg in 0..4u32 {
        let off = 0x08 + bg as usize * 2;
        let bgcnt = u16::from_le_bytes([io[off], io[off + 1]]);
        let enabled = (dispcnt >> (8 + bg)) & 1;
        if enabled != 0 || bgcnt != 0 {
            let char_base = ((bgcnt >> 2) & 3) as u32 * 0x4000;
            let screen_base = ((bgcnt >> 8) & 0x1F) as u32 * 0x800;
            println!(
                "BG{}: priority={} char={:#X} screen={:#X} enabled={}",
                bg,
                bgcnt & 3,
                char_base,
                screen_base,
                enabled
            );
        }
    }
}

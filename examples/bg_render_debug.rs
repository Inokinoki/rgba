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

    for round in 0..80 {
        gba.input.press_key(KeyState::A);
        for _ in 0..4 {
            gba.run_frame_parallel(&mut fb);
        }
        gba.input.release_key(KeyState::A);
        for _ in 0..16 {
            gba.run_frame_parallel(&mut fb);
        }
    }

    let vram = gba.mem().vram();
    let io = gba.mem().io();
    let pal = gba.mem().palette();
    let dispcnt = u16::from_le_bytes([io[0], io[1]]);

    println!("DISPCNT: {:#06X}", dispcnt);
    for bg in 0..4u32 {
        let off = 0x08 + bg as usize * 2;
        let bgcnt = u16::from_le_bytes([io[off], io[off + 1]]);
        let enabled = (dispcnt >> (8 + bg)) & 1;
        if enabled != 0 || bgcnt != 0 {
            let char_base = ((bgcnt >> 2) & 3) as u32 * 0x4000;
            let screen_base = ((bgcnt >> 8) & 0x1F) as u32 * 0x800;
            let size = (bgcnt >> 14) & 3;
            let priority = bgcnt & 3;
            let mosaic = (bgcnt >> 6) & 1;
            let color_mode = (bgcnt >> 7) & 1;
            println!(
                "BG{}: priority={} char={:#X} screen={:#X} size={} mosaic={} {}bit enabled={}",
                bg,
                priority,
                char_base,
                screen_base,
                size,
                mosaic,
                if color_mode != 0 { "256" } else { "16" },
                enabled
            );

            let scx_off = 0x10 + bg as usize * 2;
            let scx = u16::from_le_bytes([io[scx_off], io[scx_off + 1]]);
            let scy = u16::from_le_bytes([io[scx_off + 2], io[scx_off + 3]]);
            println!("  scroll: X={} Y={}", scx & 0x1FF, scy & 0x1FF);
        }
    }

    println!("\n=== Palette ===");
    for i in 0..16usize {
        let color = u16::from_le_bytes([pal[i * 2], pal[i * 2 + 1]]);
        let r = color & 0x1F;
        let g = (color >> 5) & 0x1F;
        let b = (color >> 10) & 0x1F;
        print!("  [{:2}]={:04X}({},{},{})", i, color, r, g, b);
        if i % 4 == 3 {
            println!();
        }
    }

    println!("\n=== BG map sample for enabled BGs ===");
    for bg in 0..4u32 {
        let off = 0x08 + bg as usize * 2;
        let bgcnt = u16::from_le_bytes([io[off], io[off + 1]]);
        let enabled = (dispcnt >> (8 + bg)) & 1;
        if enabled != 0 {
            let screen_base = ((bgcnt >> 8) & 0x1F) as u32 * 0x800;
            let char_base = ((bgcnt >> 2) & 3) as u32 * 0x4000;
            println!(
                "\nBG{} map at {:#X} (char_base={:#X}):",
                bg, screen_base, char_base
            );

            for y in 0..5u32 {
                for x in 0..10u32 {
                    let i = y * 32 + x;
                    let map_off = screen_base as usize + i as usize * 2;
                    let entry = u16::from_le_bytes([vram[map_off], vram[map_off + 1]]);
                    let tile = entry & 0x3FF;
                    let pal_bank = (entry >> 12) & 0xF;
                    let hflip = (entry >> 10) & 1;
                    let vflip = (entry >> 11) & 1;
                    print!("{:4}", tile);
                }
                println!();
            }

            let tile_sample = {
                let i = 0;
                let map_off = screen_base as usize + i * 2;
                let entry = u16::from_le_bytes([vram[map_off], vram[map_off + 1]]);
                entry & 0x3FF
            };

            let tile_data_off = char_base as usize + tile_sample as usize * 32;
            print!("  Tile {} data: ", tile_sample);
            for b in 0..32 {
                if tile_data_off + b < vram.len() {
                    print!("{:02X}", vram[tile_data_off + b]);
                }
            }
            println!();

            let tile_empty = tile_data_off..tile_data_off + 32;
            let all_zero =
                (tile_data_off..tile_data_off + 32).all(|i| i < vram.len() && vram[i] == 0);
            println!(
                "  Tile {} is {}",
                tile_sample,
                if all_zero { "EMPTY" } else { "HAS DATA" }
            );
        }
    }

    println!("\n=== Tile count ===");
    let mut nonzero = 0;
    for tile in 0..1024u32 {
        let base = tile as usize * 32;
        let mut has_data = false;
        for b in 0..32 {
            if base + b < vram.len() && vram[base + b] != 0 {
                has_data = true;
                break;
            }
        }
        if has_data {
            nonzero += 1;
        }
    }
    println!("{} nonzero tiles in char block 0", nonzero);
}

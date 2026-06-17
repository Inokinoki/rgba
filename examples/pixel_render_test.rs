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

    gba.sync_ppu_full();

    let io = gba.mem().io();
    let dispcnt = u16::from_le_bytes([io[0], io[1]]);
    let ppu = &gba.ppu();

    println!("PPU state after sync:");
    println!("  DISPCNT: {:#06X}", ppu.get_dispcnt());
    for bg in 0..4 {
        let bgcnt = ppu.get_bgcnt(bg);
        let enabled = (dispcnt >> (8 + bg)) & 1;
        let tile_base = ppu.get_bg_tile_base(bg);
        let map_base = ppu.get_bg_map_base(bg);
        let hofs = ppu.get_bg_hofs(bg);
        let vofs = ppu.get_bg_vofs(bg);
        let priority = ppu.get_bg_priority(bg);
        let mosaic = (bgcnt >> 6) & 1;
        println!(
            "  BG{}: cnt={:#06X} en={} pri={} tile={:#X} map={:#X} hofs={} vofs={} mosaic={}",
            bg, bgcnt, enabled, priority, tile_base, map_base, hofs, vofs, mosaic
        );
    }

    println!("\n=== Render test for pixel (120, 80) ===");
    let ppu = gba.ppu();
    let vram = ppu.vram();

    for bg in 0..4 {
        let bgcnt = ppu.get_bgcnt(bg);
        let enabled = (dispcnt >> (8 + bg)) & 1;
        if enabled == 0 {
            continue;
        }

        let tile_base = ppu.get_bg_tile_base(bg) as usize;
        let map_base = ppu.get_bg_map_base(bg) as usize;
        let hofs = ppu.get_bg_hofs(bg) as u32;
        let vofs = ppu.get_bg_vofs(bg) as u32;
        let mosaic = (bgcnt >> 6) & 1;

        let x = 120u16;
        let y = 80u16;
        let bg_x = ((x as u32 + hofs) % 256) as u16;
        let bg_y = ((y as u32 + vofs) % 256) as u16;

        let (bg_x, bg_y) = if mosaic != 0 {
            let mh = (ppu.bg_mosaic & 0xF) as u16;
            let mv = ((ppu.bg_mosaic >> 4) & 0xF) as u16;
            let bx = if mh > 0 {
                (bg_x / (mh + 1)) * (mh + 1)
            } else {
                bg_x
            };
            let by = if mv > 0 {
                (bg_y / (mv + 1)) * (mv + 1)
            } else {
                bg_y
            };
            (bx, by)
        } else {
            (bg_x, bg_y)
        };

        let tile_x = bg_x / 8;
        let tile_y = bg_y / 8;
        let pixel_x = bg_x % 8;
        let pixel_y = bg_y % 8;

        let entry_offset = map_base + (tile_y as usize * 32 + tile_x as usize) * 2;
        let entry = u16::from_le_bytes([vram[entry_offset], vram[entry_offset + 1]]);
        let tile_num = entry & 0x3FF;
        let palette_num = (entry >> 12) & 0xF;

        let tile_data_off = tile_base + tile_num as usize * 32;
        let row_off = tile_data_off + pixel_y as usize * 4;
        let byte_val = vram[row_off + pixel_x as usize / 2];
        let color_index = if pixel_x % 2 == 0 {
            byte_val & 0x0F
        } else {
            byte_val >> 4
        };

        println!("  BG{}: bg_x={} bg_y={} tile=({},{}) pixel=({},{}) entry={:#06X} tile_num={} pal={} color_idx={}",
            bg, bg_x, bg_y, tile_x, tile_y, pixel_x, pixel_y, entry, tile_num, palette_num, color_index);

        if tile_data_off + 32 <= vram.len() {
            print!("    tile data: ");
            for b in 0..32 {
                print!("{:02X}", vram[tile_data_off + b]);
            }
            println!();
        } else {
            println!("    tile data OUT OF RANGE ({:#X})", tile_data_off);
        }
    }

    println!("\n=== Full pixel render at (120, 80) ===");
    let color = gba.get_pixel_tile_mode(120, 80);
    let r = (color & 0x1F) as u32 * 255 / 31;
    let g = ((color >> 5) & 0x1F) as u32 * 255 / 31;
    let b = ((color >> 10) & 0x1F) as u32 * 255 / 31;
    println!("  color={:#06X} rgb=({},{},{})", color, r, g, b);
}

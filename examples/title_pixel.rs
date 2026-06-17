use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    gba.sync_ppu_full();
    let ppu = gba.ppu();
    let vram = ppu.vram();
    let palette = gba.mem().palette();

    println!("=== Title screen BG tile data ===");
    let test_tiles = [394, 403, 412, 420, 473, 482, 491, 499, 611, 629];

    for &t in &test_tiles {
        let off = t as usize * 32;
        let mut nonzero = 0;
        for b in 0..32 {
            if vram[off + b] != 0 {
                nonzero += 1;
            }
        }
        if nonzero == 0 {
            println!("Tile {}: ALL ZERO", t);
        } else {
            print!("Tile {} (off={:#X}): ", t, off);
            for b in 0..32 {
                print!("{:02X}", vram[off + b]);
            }
            println!();
        }
    }

    println!("\n=== Palette entries (BG, first 16) ===");
    for i in 0..16 {
        let off = i * 2;
        let c = u16::from_le_bytes([palette[off], palette[off + 1]]);
        let r = c & 0x1F;
        let g = (c >> 5) & 0x1F;
        let b = (c >> 10) & 0x1F;
        print!("[{:2}]={:#06X}({},{},{}) ", i, c, r, g, b);
        if i % 4 == 3 {
            println!();
        }
    }
    println!();

    println!("\n=== Check pixel at (120, 80) on title screen ===");
    let bgcnt = ppu.get_bgcnt(3);
    let hofs = ppu.get_bg_hofs(3);
    let vofs = ppu.get_bg_vofs(3);
    let map_base = ppu.get_bg_map_base(3) as usize;
    println!(
        "BG3: cnt={:#06X} hofs={} vofs={} map={:#X}",
        bgcnt, hofs, vofs, map_base
    );

    let x = 120u16;
    let y = 80u16;
    let bg_x = ((x as u32 + hofs as u32) % 256) as u16;
    let bg_y = ((y as u32 + vofs as u32) % 256) as u16;
    let tile_x = bg_x / 8;
    let tile_y = bg_y / 8;
    let pixel_x = bg_x % 8;
    let pixel_y = bg_y % 8;

    let entry_off = map_base + (tile_y as usize * 32 + tile_x as usize) * 2;
    let entry = u16::from_le_bytes([vram[entry_off], vram[entry_off + 1]]);
    let tile_num = entry & 0x3FF;
    let pal_num = (entry >> 12) & 0xF;
    println!(
        "  bg_x={} bg_y={} tile=({},{})) pixel=({},{}) entry={:#06X} tile_num={} pal={}",
        bg_x, bg_y, tile_x, tile_y, pixel_x, pixel_y, entry, tile_num, pal_num
    );

    let tile_off = tile_num as usize * 32;
    let row_off = tile_off + pixel_y as usize * 4;
    let byte_val = vram[row_off + pixel_x as usize / 2];
    let color_idx = if pixel_x % 2 == 0 {
        byte_val & 0x0F
    } else {
        byte_val >> 4
    };
    let pal_index = pal_num as usize * 16 + color_idx as usize;
    let pal_off = pal_index * 2;
    let color = u16::from_le_bytes([palette[pal_off], palette[pal_off + 1]]);
    println!(
        "  byte={:#04X} color_idx={} pal_idx={} color={:#06X}",
        byte_val, color_idx, pal_index, color
    );

    let final_color = gba.get_pixel_tile_mode(x, y);
    println!("  final pixel color: {:#06X}", final_color);
}

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
    for _ in 0..4 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input.release_key(KeyState::START);
    for _ in 0..180 {
        gba.run_frame_parallel(&mut fb);
    }

    let vram = gba.mem.vram();
    let io = gba.mem.io();
    let dispcnt = u16::from_le_bytes([io[0], io[1]]);
    let bg0cnt = u16::from_le_bytes([io[8], io[9]]);

    // Check tile 1023
    let char_base = 0;
    let tile_1023_offset = char_base + 1023 * 32;
    println!("=== Tile 1023 at offset {:05X} ===", tile_1023_offset);
    let mut all_zero = true;
    for j in 0..32 {
        if vram[tile_1023_offset + j] != 0 {
            all_zero = false;
        }
    }
    if all_zero {
        println!("  ALL ZEROS (transparent)");
    } else {
        print!("  DATA: ");
        for j in 0..32 {
            print!("{:02X} ", vram[tile_1023_offset + j]);
        }
        println!();
    }

    // Manually trace BG0 pixel (0, 0)
    println!("\n=== BG0 pixel trace for (0, 0) ===");
    let bg0_screen_base = ((bg0cnt >> 8) & 0x1F) as usize * 0x800;
    let se = u16::from_le_bytes([vram[bg0_screen_base], vram[bg0_screen_base + 1]]);
    let tile = se & 0x3FF;
    let hflip = (se >> 10) & 1;
    let vflip = (se >> 11) & 1;
    let pal = (se >> 12) & 0xF;
    println!(
        "SE[0,0] = {:04X} (tile={} h={} v={} pal={})",
        se, tile, hflip, vflip, pal
    );

    let tile_offset = char_base + tile as usize * 32;
    let byte0 = vram[tile_offset];
    let pix0_0 = byte0 & 0x0F;
    println!(
        "Tile {} row 0 byte: {:02X}, pixel(0,0) color_idx={}",
        tile, byte0, pix0_0
    );

    let pal_ram = gba.mem.palette();
    if pix0_0 != 0 {
        let pal_offset = (pal as usize * 16 + pix0_0 as usize) * 2;
        let color = u16::from_le_bytes([pal_ram[pal_offset], pal_ram[pal_offset + 1]]);
        println!("Palette[{}][{}] = {:04X}", pal, pix0_0, color);
    }

    // BG3 pixel trace
    let bg3cnt = u16::from_le_bytes([io[0xE], io[0xF]]);
    let bg3_screen_base = ((bg3cnt >> 8) & 0x1F) as usize * 0x800;
    let se3 = u16::from_le_bytes([vram[bg3_screen_base], vram[bg3_screen_base + 1]]);
    let tile3 = se3 & 0x3FF;
    let pal3 = (se3 >> 12) & 0xF;
    println!("\nBG3 SE[0,0] = {:04X} (tile={} pal={})", se3, tile3, pal3);
    let tile3_offset = char_base + tile3 as usize * 32;
    let byte3_0 = vram[tile3_offset];
    println!("BG3 tile {} pixel(0,0) color_idx={}", tile3, byte3_0 & 0x0F);

    // Check what get_pixel_tile_mode actually renders
    let result = gba.get_pixel_tile_mode(0, 0);
    let r = ((result & 0x1F) as u32 * 255 / 31) << 16;
    let g = (((result >> 5) & 0x1F) as u32 * 255 / 31) << 8;
    let b = ((result >> 10) & 0x1F) as u32 * 255 / 31;
    println!(
        "\nget_pixel_tile_mode(0,0) = {:04X} -> fb {:08X}",
        result,
        r | g | b
    );
    println!("Framebuffer[0] = {:08X}", fb[0]);

    // Trace through get_bg_pixel for each BG layer at (0,0)
    println!("\n=== Per-BG pixel at (0,0) ===");
    for bg in 0..4 {
        let cnt = [io[8], io[9], io[0xC], io[0xD], io[0xE], io[0xF]];
        let bg_cnt_off = 8 + bg * 2;
        let bgcnt = u16::from_le_bytes([io[bg_cnt_off], io[bg_cnt_off + 1]]);
        let enabled = (dispcnt >> (8 + bg)) & 1;
        let pri = bgcnt & 3;
        if enabled == 0 {
            println!("BG{}: disabled", bg);
            continue;
        }

        let screen_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
        let se = u16::from_le_bytes([vram[screen_base], vram[screen_base + 1]]);
        let tile = se & 0x3FF;
        let pal = (se >> 12) & 0xF;
        let tile_off = char_base + tile as usize * 32;
        let byte = vram[tile_off];
        let cidx = byte & 0x0F;
        println!(
            "BG{}: pri={} tile={} pal={} cidx={}",
            bg, pri, tile, pal, cidx
        );

        if cidx != 0 {
            let pal_off = (pal as usize * 16 + cidx as usize) * 2;
            let c = u16::from_le_bytes([pal_ram[pal_off], pal_ram[pal_off + 1]]);
            println!("  -> color {:04X}", c);
        } else {
            println!("  -> transparent (cidx=0)");
        }
    }

    // Print palette 11
    println!("\n=== Palette 11 (BG0) ===");
    for i in 0..16 {
        let off = (11 * 16 + i) * 2;
        let c = u16::from_le_bytes([pal_ram[off], pal_ram[off + 1]]);
        let r = c & 0x1F;
        let g = (c >> 5) & 0x1F;
        let b = (c >> 10) & 0x1F;
        println!("  PAL[11][{}] = {:04X} (R{} G{} B{})", i, c, r, g, b);
    }
}

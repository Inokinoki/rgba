use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..500u32 {
        gba.run_frame_parallel(&mut fb);
    }

    gba.sync_ppu_full();

    let ppu = gba.ppu();

    // Debug sprite 6: y=116, x=100, w=16, h=16, tile=768
    let sprite = 6;
    let enabled = ppu.sprite_is_enabled(sprite);
    let is_window = ppu.sprite_is_window(sprite);
    let (w, h) = ppu.sprite_dimensions(sprite);
    let sx = ppu.sprite_x(sprite);
    let sy = ppu.sprite_y(sprite);
    let tile = ppu.sprite_tile(sprite);
    let is_256 = ppu.sprite_is_256color(sprite);
    let pal = ppu.sprite_palette(sprite);

    println!(
        "Sprite 6: enabled={} window={} y={} x={} w={} h={} tile={} 256c={} pal={}",
        enabled, is_window, sy, sx, w, h, tile, is_256, pal
    );

    // Check pixel at (100, 116) - should be top-left of sprite 6
    let (x, y) = (100u16, 116u16);
    let dx = x as i32 - sx;
    let dy = y as i32 - sy;
    println!(
        "  dx={} dy={} range: dx<{} dy<{}",
        dx, dy, w as i32, h as i32
    );
    println!(
        "  in range: {}",
        dx >= 0 && dx < w as i32 && dy >= 0 && dy < h as i32
    );

    // Read tile data for sprite 6
    let obj_base = 0x10000usize;
    let tile_offset = obj_base + (tile as usize * 32);
    let vram = ppu.vram();
    println!("  tile_offset=0x{:X} vram.len={}", tile_offset, vram.len());
    if tile_offset < vram.len() {
        print!("  tile data: ");
        for b in 0..32 {
            print!("{:02X} ", vram[tile_offset + b]);
        }
        println!();
    }

    // Check pixel at row 0, col 0 of the tile
    let tile_x = 0u16;
    let tile_y = 0u16;
    let pixel_x = 0u8;
    let pixel_y = 0u8;
    let actual_tile = tile + tile_y * (w / 8) + tile_x;
    println!("  actual_tile={}", actual_tile);
    let ci = ppu.get_obj_tile_pixel(actual_tile, pixel_x, pixel_y, pal, is_256);
    println!("  color_index at (0,0) = {}", ci);

    // Check palette
    let pal_data = gba.mem.palette();
    let pal_offset = if is_256 {
        0
    } else {
        (pal as usize * 16 + 128) * 2
    };
    println!("  palette offset = 0x{:X}", pal_offset);
    if pal_offset < pal_data.len() {
        let mut pal_entries = vec![];
        for i in 0..16 {
            let v = u16::from_le_bytes([
                pal_data[pal_offset + i * 2],
                pal_data[pal_offset + i * 2 + 1],
            ]);
            pal_entries.push(format!("{:04X}", v));
        }
        println!("  pal: {}", pal_entries.join(" "));
    }
}

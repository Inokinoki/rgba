use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];
    for _ in 0..240 { gba.run_frame_parallel(&mut framebuffer); }
    gba.input_mut().press_key(KeyState::START);
    for _ in 0..60 { gba.run_frame_parallel(&mut framebuffer); }
    gba.input_mut().release_key(KeyState::START);
    for _ in 0..120 { gba.run_frame_parallel(&mut framebuffer); }
    for _ in 0..100 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..3 { gba.run_frame_parallel(&mut framebuffer); }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..12 { gba.run_frame_parallel(&mut framebuffer); }
    }

    let ppu = gba.ppu();
    let vram = ppu.vram();

    // BG1: map_base=0xF000, tile_base=0, hofs=2, vofs=6, size=0
    // Pixel (2,6): bg_x = (2+2)%256 = 4, bg_y = (6+6)%256 = 12
    // tile_x = 0, tile_y = 1, pixel_x = 4, pixel_y = 4
    let entry = ppu.get_screen_entry(0xF000, 0, 1, 0, 256/8, 256/8);
    println!("BG1 screen entry at tile (0,1): {:#X}", entry);

    // Also get the actual rendered pixel
    let color = gba.get_pixel_tile_mode(2, 6);
    let r = color & 0x1F;
    let g = (color >> 5) & 0x1F;
    let b = (color >> 10) & 0x1F;
    println!("get_pixel_tile_mode(2,6) = {:#X} R={} G={} B={}", color, r, g, b);

    // Let me trace the full rendering for pixel (2,6):
    println!("\n=== Full pixel trace for (2,6) ===");
    for bg in 0..4u32 {
        let bgcnt = ppu.get_bgcnt(bg as usize);
        let bg_size = (bgcnt >> 14) & 3;
        let tile_base = ppu.get_bg_tile_base(bg as usize) as usize;
        let map_base = ppu.get_bg_map_base(bg as usize) as usize;
        let hofs = ppu.get_bg_hofs(bg as usize) as u32;
        let vofs = ppu.get_bg_vofs(bg as usize) as u32;
        let width: u16 = match bg_size { 0=>256, 1=>512, 2=>256, 3=>512, _=>256 };
        let height: u16 = match bg_size { 0=>256, 1=>256, 2=>512, 3=>512, _=>256 };

        let bg_x = ((2u32 + hofs) % width as u32) as u16;
        let bg_y = ((6u32 + vofs) % height as u32) as u16;
        let tile_x = bg_x / 8;
        let tile_y = bg_y / 8;
        let pixel_x = bg_x % 8;
        let pixel_y = bg_y % 8;

        let entry = ppu.get_screen_entry(map_base, tile_x, tile_y, bg_size, width/8, height/8);
        let (tile_num, flip_h, flip_v, pal, _) = rgba::Ppu::parse_screen_entry(entry);

        let is_8bpp = (bgcnt & 0x80) != 0;
        let color_idx = if is_8bpp {
            ppu.get_tile_pixel_8bpp(tile_base, tile_num, pixel_x as u8, pixel_y as u8, flip_h, flip_v)
        } else {
            ppu.get_tile_pixel_4bpp(tile_base, tile_num, pixel_x as u8, pixel_y as u8, pal, flip_h, flip_v)
        };

        let priority = bgcnt & 3;
        println!("  BG{}: priority={} entry={:#X} tile={} pal={} flip_h={} flip_v={} px=({},{}) color_idx={}",
            bg, priority, entry, tile_num, pal, flip_h, flip_v, pixel_x, pixel_y, color_idx);
    }
}

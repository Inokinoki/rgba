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
    for round in 0..200 {
        if round % 10 < 7 {
            gba.input_mut().press_key(KeyState::A);
            for _ in 0..3 { gba.run_frame_parallel(&mut framebuffer); }
            gba.input_mut().release_key(KeyState::A);
        } else {
            let dir = match round % 4 {
                0 => KeyState::UP, 1 => KeyState::DOWN,
                2 => KeyState::LEFT, _ => KeyState::RIGHT,
            };
            gba.input_mut().press_key(dir);
            for _ in 0..3 { gba.run_frame_parallel(&mut framebuffer); }
            gba.input_mut().release_key(dir);
        }
        for _ in 0..12 { gba.run_frame_parallel(&mut framebuffer); }

        if round == 75 || round == 78 {
            // After run_frame_parallel, the framebuffer was just rendered
            // Check what get_pixel_tile_mode returns NOW (should match framebuffer)
            let c_api = gba.get_pixel_tile_mode(120, 80);
            let c_fb = framebuffer[80 * 240 + 120];
            let fb_r = ((c_fb >> 16) & 0xFF) as u32;
            let fb_g = ((c_fb >> 8) & 0xFF) as u32;
            let fb_b = (c_fb & 0xFF) as u32;
            // Convert framebuffer color to 5-bit
            let fb_5bit_r = fb_r * 31 / 255;
            let fb_5bit_g = fb_g * 31 / 255;
            let fb_5bit_b = fb_b * 31 / 255;
            let fb_555 = (fb_5bit_b as u16) << 10 | (fb_5bit_g as u16) << 5 | fb_5bit_r as u16;

            println!("Round {}:", round);
            println!("  API get_pixel(120,80) = {:#X}", c_api);
            println!("  Framebuffer at (120,80) = #{:02X}{:02X}{:02X} => 5bit={:#X}", fb_r, fb_g, fb_b, fb_555);

            // Trace BG layers at this pixel
            let ppu = gba.ppu();
            for bg in 1..4 {
                let bgcnt = ppu.get_bgcnt(bg);
                let tile_base = ppu.get_bg_tile_base(bg) as usize;
                let map_base = ppu.get_bg_map_base(bg) as usize;
                let hofs = ppu.get_bg_hofs(bg) as u32;
                let vofs = ppu.get_bg_vofs(bg) as u32;
                let bg_x = ((120u32 + hofs) % 256) as u16;
                let bg_y = ((80u32 + vofs) % 256) as u16;
                let tile_x = bg_x / 8;
                let tile_y = bg_y / 8;
                let pixel_x = bg_x % 8;
                let pixel_y = bg_y % 8;
                let entry = ppu.get_screen_entry(map_base, tile_x, tile_y, 0, 32, 32);
                let (tile_num, flip_h, flip_v, pal, _) = rgba::Ppu::parse_screen_entry(entry);
                let cidx = ppu.get_tile_pixel_4bpp(tile_base, tile_num, pixel_x as u8, pixel_y as u8, pal, flip_h, flip_v);
                let pal_color = if cidx != 0 {
                    gba.get_palette_color(0, (pal * 16 + cidx as u16))
                } else { 0 };
                println!("  BG{}: tile={}({},{}) px=({},{}) cidx={} pal_color={:#X}", 
                    bg, tile_num, tile_x, tile_y, pixel_x, pixel_y, cidx, pal_color);
            }
        }
    }
}

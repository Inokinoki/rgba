use rgba::Gba;
use rgba::KeyState;

fn trace_pixel(gba: &mut Gba, x: u16, y: u16) -> String {
    let mut result = String::new();
    let ppu = gba.ppu();
    let mode = ppu.get_display_mode();
    let dispcnt = ppu.get_dispcnt();
    
    result.push_str(&format!("  DISPCNT={:#X} mode={}\n", dispcnt, mode));
    
    for bg in 0..4usize {
        let enabled = (dispcnt >> (8 + bg)) & 1 != 0;
        if !enabled { continue; }
        
        let bgcnt = ppu.get_bgcnt(bg);
        let tile_base = ppu.get_bg_tile_base(bg) as usize;
        let map_base = ppu.get_bg_map_base(bg) as usize;
        let hofs = ppu.get_bg_hofs(bg) as u32;
        let vofs = ppu.get_bg_vofs(bg) as u32;
        let bg_size = (bgcnt >> 14) & 3;
        let width: u16 = match bg_size { 0=>256, 1=>512, 2=>256, 3=>512, _=>256 };
        let height: u16 = match bg_size { 0=>256, 1=>256, 2=>512, 3=>512, _=>256 };
        
        let bg_x = ((x as u32 + hofs) % width as u32) as u16;
        let bg_y = ((y as u32 + vofs) % height as u32) as u16;
        let tile_x = bg_x / 8;
        let tile_y = bg_y / 8;
        let pixel_x = bg_x % 8;
        let pixel_y = bg_y % 8;
        
        let entry = ppu.get_screen_entry(map_base, tile_x, tile_y, bg_size, width/8, height/8);
        let (tile_num, flip_h, flip_v, pal, _) = rgba::Ppu::parse_screen_entry(entry);
        
        let vram = ppu.vram();
        let tile_off = tile_base + (tile_num as usize) * 32;
        let row_off = tile_off + (pixel_y as usize) * 4;
        let actual_byte = if tile_off + 32 <= vram.len() { vram[row_off + (pixel_x as usize) / 2] } else { 0 };
        
        let cidx = ppu.get_tile_pixel_4bpp(tile_base, tile_num, pixel_x as u8, pixel_y as u8, pal, flip_h, flip_v);
        
        result.push_str(&format!("  BG{}: en=1 pri={} entry={:#X} tile={} pal={} flip=({},{}) px=({},{}) byte={:#X} cidx={}\n",
            bg, bgcnt & 3, entry, tile_num, pal, flip_h, flip_v, pixel_x, pixel_y, actual_byte, cidx));
    }
    result
}

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
            println!("=== Round {} pixel (10,10) ===", round);
            print!("{}", trace_pixel(&mut gba, 10, 10));
            let c = gba.get_pixel_tile_mode(10, 10);
            let fb = framebuffer[10 * 240 + 10];
            println!("  API: {:#X}  FB: #{:02X}{:02X}{:02X}", c, (fb>>16)&0xFF, (fb>>8)&0xFF, fb&0xFF);
        }
    }
}

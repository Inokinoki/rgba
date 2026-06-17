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
            let mem_dispcnt = gba.mem_mut().read_half(0x0400_0000);
            let ppu_dispcnt = gba.ppu().get_dispcnt();
            let ppu = gba.ppu();
            println!("Round {}:", round);
            println!("  MEM DISPCNT: {:#X}", mem_dispcnt);
            println!("  PPU DISPCNT: {:#X}", ppu_dispcnt);
            for bg in 0..4 {
                println!("  BG{} PPU: bgcnt={:#X} tb={:#X} mb={:#X} ho={} vo={}",
                    bg, ppu.get_bgcnt(bg), ppu.get_bg_tile_base(bg),
                    ppu.get_bg_map_base(bg), ppu.get_bg_hofs(bg), ppu.get_bg_vofs(bg));
            }
            let io = gba.mem().io();
            for bg in 0..4 {
                let off = 0x08 + bg * 2;
                let bgcnt = u16::from_le_bytes([io[off], io[off+1]]);
                let hofs = u16::from_le_bytes([io[0x10+bg*4], io[0x11+bg*4]]) & 0x1FF;
                let vofs = u16::from_le_bytes([io[0x12+bg*4], io[0x13+bg*4]]) & 0x1FF;
                println!("  BG{} MEM: bgcnt={:#X} ho={} vo={}", bg, bgcnt, hofs, vofs);
            }
            // Check palette differences
            let pal = gba.mem().palette();
            let mut pal_nonzero = 0;
            for i in 0..512 {
                if pal[i] != 0 { pal_nonzero += 1; }
            }
            println!("  Palette non-zero bytes: {}", pal_nonzero);
            // Check pixel at (120, 80) - center of screen
            let c = gba.get_pixel_tile_mode(120, 80);
            println!("  Pixel(120,80): {:#X} R={} G={} B={}", c, c&0x1F, (c>>5)&0x1F, (c>>10)&0x1F);
        }
    }
}

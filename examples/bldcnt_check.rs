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
            let io = gba.mem().io();
            let bldcnt = u16::from_le_bytes([io[0x50], io[0x51]]);
            let bldalpha = u16::from_le_bytes([io[0x52], io[0x53]]);
            let bldy = u16::from_le_bytes([io[0x54], io[0x55]]);
            println!("Round {}: BLDCNT={:#X} BLDALPHA={:#X} BLDY={:#X}", round, bldcnt, bldalpha, bldy);
            let blend_mode = (bldcnt >> 6) & 3;
            let first_target: Vec<u32> = (0..6).filter(|&i| (bldcnt >> i) & 1 != 0).collect();
            let second_target: Vec<u32> = (0..6).filter(|&i| (bldcnt >> (8+i)) & 1 != 0).collect();
            println!("  Mode={} 1st={:?} 2nd={:?}", blend_mode, first_target, second_target);
            let ppu = gba.ppu();
            println!("  PPU blend_mode={} alpha={:#X}", ppu.get_blend_mode(), ppu.get_blend_alpha());
        }
    }
}

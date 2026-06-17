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

    let mut pal_75 = Vec::new();
    let mut pal_78 = Vec::new();
    let mut vram_75 = Vec::new();
    let mut vram_78 = Vec::new();

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

        if round == 75 {
            pal_75 = gba.mem().palette().to_vec();
            vram_75 = gba.mem().vram().to_vec();
        }
        if round == 78 {
            pal_78 = gba.mem().palette().to_vec();
            vram_78 = gba.mem().vram().to_vec();
        }
    }

    // Compare palettes
    let mut pal_diffs = 0;
    for i in 0..pal_75.len().min(pal_78.len()) {
        if pal_75[i] != pal_78[i] {
            pal_diffs += 1;
            if pal_diffs <= 10 {
                let c75 = u16::from_le_bytes([pal_75[i & !1], pal_75[(i & !1) + 1]]);
                let c78 = u16::from_le_bytes([pal_78[i & !1], pal_78[(i & !1) + 1]]);
                println!("Pal diff at {}: r75={:02X}{:02X}({:#X}) r78={:02X}{:02X}({:#X})",
                    i, pal_75[i], pal_75.get(i+1).unwrap_or(&0), c75,
                    pal_78[i], pal_78.get(i+1).unwrap_or(&0), c78);
            }
        }
    }
    println!("Total palette differences: {}", pal_diffs);

    // Compare VRAM
    let mut vram_diffs = 0;
    for i in 0..vram_75.len().min(vram_78.len()) {
        if vram_75[i] != vram_78[i] {
            vram_diffs += 1;
            if vram_diffs <= 10 {
                println!("VRAM diff at {:#X}: r75={:02X} r78={:02X}", i, vram_75[i], vram_78[i]);
            }
        }
    }
    println!("Total VRAM differences: {}", vram_diffs);
}

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
    let palette = gba.mem().palette();
    // Check all 16 BG palettes
    for pal in 0..16 {
        let off = pal * 32;
        let mut non_zero = 0;
        let mut colors = Vec::new();
        for i in 0..16 {
            let c = u16::from_le_bytes([palette[off + i*2], palette[off + i*2 + 1]]);
            if c != 0 { non_zero += 1; }
            colors.push(c);
        }
        println!("Pal {:2}: {} non-zero {:?}", pal, non_zero, &colors[..8]);
    }
    // Check if all palettes are the same as palette 0
    let mut all_same = true;
    for pal in 1..16 {
        let off = pal * 32;
        for i in 0..32 {
            if palette[i] != palette[off + i] { all_same = false; break; }
        }
    }
    println!("\nAll palettes identical to pal 0: {}", all_same);
}

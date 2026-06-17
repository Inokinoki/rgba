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
    println!("Palette 0:");
    for i in 0..16 {
        let off = i * 2;
        let c = u16::from_le_bytes([palette[off], palette[off+1]]);
        if c != 0 { println!("  [{}] {:#06X} R={} G={} B={}", i, c, c&0x1F, (c>>5)&0x1F, (c>>10)&0x1F); }
    }
    println!("Palette 4:");
    for i in 0..16 {
        let off = (4*16+i)*2;
        let c = u16::from_le_bytes([palette[off], palette[off+1]]);
        if c != 0 { println!("  [{}] {:#06X} R={} G={} B={}", i, c, c&0x1F, (c>>5)&0x1F, (c>>10)&0x1F); }
    }
    // Color for BG2 at pixel (2,6): pal 0, color_idx 5
    let c = gba.get_palette_color(0, 5);
    println!("\nget_palette_color(0, 5) = {:#X}", c);
    // Color for BG3: pal 4, color_idx 9
    let c = gba.get_palette_color(0, 4*16+9);
    println!("get_palette_color(0, 4*16+9) = {:#X}", c);
}

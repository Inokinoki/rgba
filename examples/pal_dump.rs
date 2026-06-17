use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];
    for _ in 0..240 { gba.run_frame_parallel(&mut framebuffer); }
    gba.input.press_key(KeyState::START);
    for _ in 0..60 { gba.run_frame_parallel(&mut framebuffer); }
    gba.input.release_key(KeyState::START);
    for _ in 0..120 { gba.run_frame_parallel(&mut framebuffer); }
    for round in 0..100 {
        gba.input.press_key(KeyState::A);
        for _ in 0..3 { gba.run_frame_parallel(&mut framebuffer); }
        gba.input.release_key(KeyState::A);
        for _ in 0..12 { gba.run_frame_parallel(&mut framebuffer); }
    }

    println!("BG Palette (256 colors, 16 palettes x 16 colors):");
    for pal in 0..16 {
        let mut line = format!("  Pal {:2}:", pal);
        for i in 0..16 {
            let color = gba.mem().read_palette_color(0, (pal * 16 + i) as u16);
            if color != 0 {
                let r = color & 0x1F;
                let g = (color >> 5) & 0x1F;
                let b = (color >> 10) & 0x1F;
                line.push_str(&format!(" [{:2}:{:#06X}=R{}G{}B{}]", i, color, r, g, b));
            }
        }
        let has_data = (0..16).any(|i| gba.mem().read_palette_color(0, (pal * 16 + i) as u16) != 0);
        if has_data { println!("{}", line); }
    }
}

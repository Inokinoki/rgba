use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    // Run 240 frames to boot
    for _ in 0..240 { gba.run_frame_parallel(&mut framebuffer); }

    // Check exact pixel values
    let center = framebuffer[80 * 240 + 120];
    let r = (center >> 16) & 0xFF;
    let g = (center >> 8) & 0xFF;
    let b = center & 0xFF;
    println!("Center pixel: {:#010X} = RGB({},{},{})", center, r, g, b);

    // Check palette
    let pal0 = gba.get_palette_color(0, 0);
    let pr = pal0 & 0x1F;
    let pg = (pal0 >> 5) & 0x1F;
    let pb = (pal0 >> 10) & 0x1F;
    println!("Palette BG 0,0: {:#06X} = RGB555({},{},{})", pal0, pr, pg, pb);
    
    // Calculate expected framebuffer value from palette
    let er = (pr as u32 * 255 / 31);
    let eg = (pg as u32 * 255 / 31);
    let eb = (pb as u32 * 255 / 31);
    let expected = (er << 16) | (eg << 8) | eb;
    println!("Expected framebuffer: {:#010X} = RGB({},{},{})", expected, er, eg, eb);

    // All unique colors in framebuffer
    let mut colors = std::collections::HashMap::new();
    for &p in &framebuffer {
        *colors.entry(p).or_insert(0u32) += 1;
    }
    println!("Unique colors: {}", colors.len());
    for (&c, &n) in &colors {
        let cr = (c >> 16) & 0xFF;
        let cg = (c >> 8) & 0xFF;
        let cb = c & 0xFF;
        println!("  {:#010X} = RGB({},{},{}) : {} pixels", c, cr, cg, cb, n);
    }

    // Now advance through game and check again
    gba.input.press_key(KeyState::START);
    for _ in 0..60 { gba.run_frame_parallel(&mut framebuffer); }
    gba.input.release_key(KeyState::START);
    for _ in 0..120 { gba.run_frame_parallel(&mut framebuffer); }

    for round in 0..200 {
        gba.input.press_key(KeyState::A);
        for _ in 0..3 { gba.run_frame_parallel(&mut framebuffer); }
        gba.input.release_key(KeyState::A);
        for _ in 0..12 { gba.run_frame_parallel(&mut framebuffer); }
    }

    // Check framebuffer after deep play
    let mut colors2 = std::collections::HashMap::new();
    for &p in &framebuffer {
        *colors2.entry(p).or_insert(0u32) += 1;
    }
    println!("\nAfter 200 rounds - Unique colors: {}", colors2.len());
    let mut sorted: Vec<_> = colors2.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    for (&c, &n) in sorted.iter().take(5) {
        let cr = (c >> 16) & 0xFF;
        let cg = (c >> 8) & 0xFF;
        let cb = c & 0xFF;
        println!("  {:#010X} = RGB({},{},{}) : {} pixels ({:.1}%)", c, cr, cg, cb, n, n as f64/38400.0*100.0);
    }
}

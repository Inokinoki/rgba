use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];

    for _ in 0..240 {
        gba.run_frame_parallel(&mut framebuffer);
    }
    gba.input_mut().press_key(KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut framebuffer);
    }
    gba.input_mut().release_key(KeyState::START);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    for _ in 0..40 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..5 {
            gba.run_frame_parallel(&mut framebuffer);
        }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..55 {
            gba.run_frame_parallel(&mut framebuffer);
        }
    }

    // Read window registers from IO memory
    let win0h = gba.mem_read_word(0x04000040) as u16;
    let win0v = gba.mem_read_word(0x04000042) as u16;
    let win1h = gba.mem_read_word(0x04000044) as u16;
    let win1v = gba.mem_read_word(0x04000046) as u16;
    let winin = gba.mem_read_word(0x04000048) as u16;
    let winout = gba.mem_read_word(0x0400004A) as u16;

    println!("=== Window Registers ===");
    println!(
        "WIN0_H: 0x{:04X} (left={}, right={})",
        win0h,
        win0h & 0xFF,
        (win0h >> 8) & 0xFF
    );
    println!(
        "WIN0_V: 0x{:04X} (top={}, bottom={})",
        win0v,
        win0v & 0xFF,
        (win0v >> 8) & 0xFF
    );
    println!(
        "WIN1_H: 0x{:04X} (left={}, right={})",
        win1h,
        win1h & 0xFF,
        (win1h >> 8) & 0xFF
    );
    println!(
        "WIN1_V: 0x{:04X} (top={}, bottom={})",
        win1v,
        win1v & 0xFF,
        (win1v >> 8) & 0xFF
    );
    println!(
        "WININ:  0x{:04X} (WIN0={}, WIN1={})",
        winin,
        winin & 0x1F,
        (winin >> 8) & 0x1F
    );
    println!("WINOUT: 0x{:04X} (outside={})", winout, winout & 0x1F);

    let dispcnt = gba.mem_read_word(0x04000000) as u16;
    let dispcnt_ppu = gba.ppu().get_dispcnt();
    println!("\n=== DISPCNT ===");
    println!("Memory: 0x{:04X}", dispcnt);
    println!("PPU:    0x{:04X}", dispcnt_ppu);
    println!("Diff:   0x{:04X}", dispcnt & !dispcnt_ppu);

    println!("\n=== Window Visibility ===");
    let win_vis = gba.ppu().get_window_visibility(120, 80);
    println!("Visibility at (120,80): 0x{:04X}", win_vis);
    println!(
        "BG0: {}, BG1: {}, BG2: {}, BG3: {}, OBJ: {}",
        (win_vis >> 0) & 1,
        (win_vis >> 1) & 1,
        (win_vis >> 2) & 1,
        (win_vis >> 3) & 1,
        (win_vis >> 4) & 1
    );

    // Check multiple points
    for &(x, y) in &[(0, 0), (120, 80), (239, 159)] {
        let v = gba.ppu().get_window_visibility(x, y);
        println!("  ({},{}): vis=0x{:04X}", x, y, v);
    }

    // Check BG3 rendering directly
    println!("\n=== BG3 State ===");
    let bgcnt = gba.ppu().get_bgcnt(3);
    println!("BGCNT: 0x{:04X}", bgcnt);
    println!("Priority: {}", bgcnt & 0x3);
    println!("Tile base: 0x{:04X}", ((bgcnt >> 2) & 0x3) * 0x4000);
    println!("Map base: 0x{:04X}", ((bgcnt >> 8) & 0x1F) * 0x800);
    println!(
        "Color mode: {}bpp",
        if (bgcnt >> 7) & 1 != 0 { 8 } else { 4 }
    );
    println!("Size: {}", (bgcnt >> 14) & 3);

    // Check what color is rendered at center pixel
    let color = gba.get_pixel_tile_mode(120, 80);
    let r = (color & 0x1F) as u32 * 255 / 31;
    let g = ((color >> 5) & 0x1F) as u32 * 255 / 31;
    let b = ((color >> 10) & 0x1F) as u32 * 255 / 31;
    println!(
        "\nPixel (120,80): RGB555=0x{:04X} RGB=({},{},{})",
        color, r, g, b
    );

    // Check blending
    let bldcnt = gba.mem_read_word(0x04000050) as u16;
    let bldalpha = gba.mem_read_word(0x04000052) as u16;
    let bldy = gba.mem_read_word(0x04000054) as u16;
    println!("\n=== Blending ===");
    println!("BLDCNT: 0x{:04X}", bldcnt);
    println!("BLDALPHA: 0x{:04X}", bldalpha);
    println!("BLDY: 0x{:04X}", bldy);
}

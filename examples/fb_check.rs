use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    println!("Framebuffer samples (10x10 grid):");
    for y in (0..160).step_by(16) {
        for x in (0..240).step_by(24) {
            let pixel = fb[y * 240 + x];
            let b = pixel & 0xFF;
            let g = (pixel >> 8) & 0xFF;
            let r = (pixel >> 16) & 0xFF;
            print!("({:3},{:3})={:#08X}=({},{},{}) ", x, y, pixel, r, g, b);
        }
        println!();
    }

    let mut color_counts = std::collections::HashMap::new();
    for &pixel in &fb {
        *color_counts.entry(pixel).or_insert(0) += 1;
    }
    let mut sorted: Vec<_> = color_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    println!("\nTop 10 colors:");
    for (color, count) in sorted.iter().take(10) {
        let b = (*color) & 0xFF;
        let g = ((*color) >> 8) & 0xFF;
        let r = ((*color) >> 16) & 0xFF;
        println!("  {:#08X} = RGB({},{},{}) count={}", color, r, g, b, count);
    }

    let final_color = gba.get_pixel_tile_mode(120, 80);
    println!("\nget_pixel_tile_mode(120,80) = {:#06X}", final_color);
    let r = (final_color & 0x1F) as u32 * 255 / 31;
    let g = ((final_color >> 5) & 0x1F) as u32 * 255 / 31;
    let b = ((final_color >> 10) & 0x1F) as u32 * 255 / 31;
    println!("RGB888: ({},{},{})", r, g, b);
}

use rgba::Gba;
use std::fs;

fn save_screenshot(fb: &[u32], png_path: &str) {
    let ppm_path = png_path.replace(".png", ".ppm");
    let mut bytes = b"P6\n240 160\n255\n".to_vec();
    for y in 0..160 {
        for x in 0..240 {
            let pixel = fb[y * 240 + x];
            bytes.push(((pixel >> 16) & 0xFF) as u8);
            bytes.push(((pixel >> 8) & 0xFF) as u8);
            bytes.push((pixel & 0xFF) as u8);
        }
    }
    fs::write(&ppm_path, &bytes).unwrap();
    std::process::Command::new("python3")
        .args([
            "-c",
            &format!(
                "from PIL import Image; Image.open('{}').save('{}')",
                ppm_path, png_path
            ),
        ])
        .output()
        .unwrap();
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..500 {
        gba.run_frame_parallel(&mut fb);
    }
    save_screenshot(&fb, "/tmp/title500.png");

    // Find all black pixels in the framebuffer (the "black hole")
    let mut black_pixels = Vec::new();
    for y in 0..160 {
        for x in 0..240 {
            let p = fb[y * 240 + x];
            if p == 0 {
                black_pixels.push((x, y));
            }
        }
    }

    println!("Black pixels: {}", black_pixels.len());
    if !black_pixels.is_empty() {
        let min_x = black_pixels.iter().map(|p| p.0).min().unwrap();
        let max_x = black_pixels.iter().map(|p| p.0).max().unwrap();
        let min_y = black_pixels.iter().map(|p| p.1).min().unwrap();
        let max_y = black_pixels.iter().map(|p| p.1).max().unwrap();
        println!("Bounds: x=[{},{}] y=[{},{}]", min_x, max_x, min_y, max_y);

        // Find the contiguous black region
        // Count by row
        for y in min_y..=max_y {
            let row_blacks: Vec<usize> = black_pixels
                .iter()
                .filter(|p| p.1 == y)
                .map(|p| p.0)
                .collect();
            if !row_blacks.is_empty() {
                println!("  y={}: x={:?}", y, row_blacks);
            }
        }
    }

    // Check what the rendering returns for those black pixel locations
    println!("\nPixel analysis at black region:");
    let ppu = gba.ppu();
    for &(x, y) in black_pixels.iter().take(5) {
        let x16 = x as u16;
        let y16 = y as u16;
        let bg0 = gba.get_bg_pixel(ppu, 0, 0, x16, y16);
        let bg1 = gba.get_bg_pixel(ppu, 0, 1, x16, y16);
        let bg2 = gba.get_bg_pixel(ppu, 0, 2, x16, y16);
        let bg3 = gba.get_bg_pixel(ppu, 0, 3, x16, y16);
        let sprite = gba.get_sprite_pixel(ppu, x16, y16);
        let tile_mode = gba.get_pixel_tile_mode(x16, y16);
        println!(
            "  ({},{}): bg0={:?} bg1={:?} bg2={:?} bg3={:?} spr={:?} final={:04X}",
            x, y, bg0, bg1, bg2, bg3, sprite, tile_mode
        );
    }

    // Now check mGBA at same frame for comparison
    // mGBA frame 500 OAM first 10 sprites:
    // [0] a0=70F2 a1=8024 a2=0080 y=242 x=36  (off screen, y=242 > 160)
    // [1] a0=70F2 a1=8044 a2=0090 y=242 x=68
    // ...
    // [5] a0=B0F2 a1=00C4 a2=00F8 y=242 x=196
    // [6] a0=507C a1=00D0 a2=8300 y=124 x=208  (visible!)
    // [7] a0=107C a1=00E0 a2=8302 y=124 x=224

    // Check our OAM for visible sprites
    let oam = ppu.oam();
    println!("\nOur OAM visible sprites:");
    for i in 0..128 {
        let base = i * 8;
        let a0 = u16::from_le_bytes([oam[base], oam[base + 1]]);
        let a1 = u16::from_le_bytes([oam[base + 2], oam[base + 3]]);
        let a2 = u16::from_le_bytes([oam[base + 4], oam[base + 5]]);
        let y_raw = a0 & 0xFF;
        let x = a1 & 0x1FF;
        let shape = (a0 >> 14) & 3;
        let size_bits = (a1 >> 14) & 3;
        let tile_num = a2 & 0x3FF;
        let priority = (a2 >> 10) & 3;
        let pal_bank = (a2 >> 12) & 0xF;
        let is_256 = (a0 >> 13) & 1;
        let dbl = (a0 >> 9) & 1;
        let y = if y_raw >= 160 && (shape != 0 || dbl != 0) {
            y_raw
        } else {
            y_raw
        };

        // Only show potentially visible sprites
        if shape != 0 || (y_raw < 160 && y_raw > 0) {
            // Get dimensions
            let (w, h) = match (shape, size_bits) {
                (0, 0) => (8, 8),
                (0, 1) => (16, 16),
                (0, 2) => (32, 32),
                (0, 3) => (64, 64),
                (1, 0) => (16, 8),
                (1, 1) => (32, 8),
                (1, 2) => (8, 16),
                (1, 3) => (16, 32),
                (2, 0) => (8, 16),
                (2, 1) => (8, 32),
                (2, 2) => (16, 8),
                (2, 3) => (32, 8),
                _ => (8, 8),
            };
            // Affine sprites use different size
            let is_affine = (a0 >> 12) & 1 != 0;
            if y_raw < 160 + h as u16 && y_raw > 0 && x < 240 {
                println!(
                    "  [{}] y={} x={} shape={} size={} tile={} pri={} pal={} 256={} dbl={} aff={}",
                    i,
                    y_raw,
                    x,
                    shape,
                    size_bits,
                    tile_num,
                    priority,
                    pal_bank,
                    is_256,
                    dbl,
                    is_affine
                );
            }
        }
    }

    // Check mGBA state at frame 500 for comparison
    // mGBA: DISPCNT=1F40, BG0CNT=5843, BG1CNT=5A42, BG2CNT=5C41, BG3CNT=5E40
    // All HOFS=1F7 VOFS=1F7
    // Our: DISPCNT=1F40, same BGCNTs, HOFS=000 VOFS=000
    // The HOFS/VOFS differ! mGBA has 0x1F7, ours has 0x000
    // 0x1F7 = 503, which wraps to 503 % 512 = 503 for scrolling

    // Actually wait - mGBA might be showing different scroll because the timing differs
    // Let me just check our HOFS/VOFS
    println!("\nScroll offsets:");
    for bg in 0..4 {
        println!(
            "BG{}: hofs={} vofs={}",
            bg,
            ppu.get_bg_hofs(bg),
            ppu.get_bg_vofs(bg)
        );
    }
}

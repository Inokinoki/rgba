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
    for _ in 0..100 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..3 {
            gba.run_frame_parallel(&mut framebuffer);
        }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..12 {
            gba.run_frame_parallel(&mut framebuffer);
        }
    }

    let bg3cnt = gba.mem_mut().read_half(0x0400_000E);
    let bg3hofs = gba.mem_mut().read_half(0x0400_0014) & 0x1FF;
    let bg3vofs = gba.mem_mut().read_half(0x0400_0016) & 0x1FF;
    let scr_base = ((bg3cnt >> 8) & 0x1F) as u32 * 0x800;
    let tile_base = ((bg3cnt >> 2) & 0x3) as u32 * 0x4000;

    println!(
        "BG3CNT={:#06X} tile_base={:#X} scr_base={:#X} hofs={} vofs={}",
        bg3cnt, tile_base, scr_base, bg3hofs, bg3vofs
    );

    let vram = gba.mem().vram();
    let palette = gba.mem().palette();

    println!("\n=== Manually decode tile 279, palette 4 ===");
    let tile_off = (tile_base + 279 * 32) as usize;
    let pal_off = 4 * 16;
    for row in 0..8 {
        let b0 = vram[tile_off + row * 4];
        let b1 = vram[tile_off + row * 4 + 1];
        let b2 = vram[tile_off + row * 4 + 2];
        let b3 = vram[tile_off + row * 4 + 3];
        let mut line = String::new();
        for col in 0..8 {
            let lo = ((vram[tile_off + row * 4 + col / 4] >> ((col % 4) * 2)) & 3) as usize;
            let hi = ((vram[tile_off + row * 4 + col / 4 + 2] >> ((col % 4) * 2)) & 3) as usize;
            let idx = lo | (hi << 2);
            let color_off = (pal_off + idx) * 2;
            let color = u16::from_le_bytes([palette[color_off], palette[color_off + 1]]);
            if idx == 0 {
                line.push_str(" . ");
            } else {
                line.push_str(&format!(" {:02X}", idx));
            }
        }
        println!(
            "  row {}: {}  raw: {:02X} {:02X} {:02X} {:02X}",
            row, line, b0, b1, b2, b3
        );
    }

    println!("\n=== get_pixel_tile_mode output ===");
    for y in 6..14u16 {
        for x in 2..10u16 {
            let color = gba.get_pixel_tile_mode(x, y);
            let r = color & 0x1F;
            let g = (color >> 5) & 0x1F;
            let b = (color >> 10) & 0x1F;
            print!(" ({},{})={:02X}{:02X}{:02X}", x, y, r, g, b);
        }
        println!();
    }

    println!("\n=== Framebuffer at BG3 area ===");
    for y in 6..14u16 {
        for x in 2..10u16 {
            let pixel = framebuffer[(y as usize) * 240 + (x as usize)];
            let r = (pixel >> 16) & 0xFF;
            let g = ((pixel >> 8) & 0xFF);
            let b = (pixel & 0xFF);
            print!(" {:02X}{:02X}{:02X}", r, g, b);
        }
        println!();
    }

    let mut unique_colors: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    for &p in &framebuffer {
        *unique_colors.entry(p).or_insert(0) += 1;
    }
    let mut sorted: Vec<_> = unique_colors.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    println!("\n=== Top 20 framebuffer colors ===");
    for (i, (color, count)) in sorted.iter().take(20).enumerate() {
        let r = (**color >> 16) & 0xFF;
        let g = (**color >> 8) & 0xFF;
        let b = **color & 0xFF;
        println!("  [{}] #{:02X}{:02X}{:02X} count={}", i, r, g, b, count);
    }
}

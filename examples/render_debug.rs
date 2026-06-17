use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    // Check the actual framebuffer rendering more carefully
    // The FB should show the composite of all BG layers
    // Let's check pixel colors along the text area (rows 0-64, which should have BG0 tiles)

    let dispcnt = gba.mem.read_half(0x04000000);
    println!("DISPCNT: 0x{:04X}", dispcnt);
    println!("  Mode: {}", dispcnt & 7);
    println!("  BG0 enable: {}", (dispcnt >> 8) & 1);
    println!("  BG1 enable: {}", (dispcnt >> 9) & 1);
    println!("  BG2 enable: {}", (dispcnt >> 10) & 1);
    println!("  BG3 enable: {}", (dispcnt >> 11) & 1);
    println!("  OBJ enable: {}", (dispcnt >> 12) & 1);

    // Sample actual framebuffer pixels in the text area
    println!("\nFramebuffer samples at text area (row 0-8):");
    for y in [0u32, 4, 8, 16, 32, 48, 64] {
        let mut unique = std::collections::HashSet::new();
        for x in [0, 8, 16, 24, 32, 48, 64, 96, 120, 160, 200, 239] {
            let c = fb[(y * 240 + x) as usize];
            unique.insert(c & 0xFFFFFF);
        }
        println!(
            "  Row {}: {} unique colors: {:?}",
            y,
            unique.len(),
            unique
                .iter()
                .take(5)
                .map(|c| format!("0x{:06X}", c))
                .collect::<Vec<_>>()
        );
    }

    // Check PPU snapshot at frame 200 - what does it think it should render?
    // The PPU renders based on a snapshot taken at the start of each scanline.
    // Let's check if the sync is correct.

    // Check if BG0 scroll is being applied correctly
    let bg0h = gba.mem.read_half(0x04000010) & 0x1FF;
    let bg0v = gba.mem.read_half(0x04000012) & 0x1FF;
    println!("\nBG0 scroll: h={}, v={}", bg0h, bg0v);

    // The visible area at screen x=0 with hscroll=224:
    // map_x = (0 + 224) % 256 = 224 -> tile_col = 28
    // Tile 28 in row 0 has entry 0xB1E1 -> tile 481, pal 11
    // Tile 481 has all pixels = 2 in palette 11
    // Palette 11, color 2 = 0x7E80 -> RGB(0,28,25) which is dark blue-green
    // But the framebuffer shows bright sky blue at (0,0)

    // This means BG3 (priority 0, highest) or another layer is covering BG0
    // Let's check BG3 tile map more carefully
    let vram = gba.mem.vram();

    // BG3: tb=0xF000, cb=0x0, pri=0
    // With hscroll=0x00E0=224
    let bg3h = gba.mem.read_half(0x0400001C) & 0x1FF;
    let bg3v = gba.mem.read_half(0x0400001E) & 0x1FF;
    println!("BG3 scroll: h={}, v={}", bg3h, bg3v);

    // What does BG3 render at (0,0)?
    let mx3 = (0 + bg3h as usize) % 256;
    let my3 = (0 + bg3v as usize) % 256;
    let tc3 = mx3 / 8;
    let tr3 = my3 / 8;
    let eo3 = 0xF000 + (tr3 * 64 + tc3) * 2;
    let te3 = u16::from_le_bytes([vram[eo3], vram[eo3 + 1]]);
    let ti3 = te3 & 0x3FF;
    println!("BG3 at (0,0): tile={} (0x{:04X})", ti3, te3);

    // Check if tile 0x3FF really has all zeros
    let mut all_zero = true;
    for i in 0..32 {
        if vram[0x3FF * 32 + i] != 0 {
            all_zero = false;
            break;
        }
    }
    println!("Tile 0x3FF all zero: {}", all_zero);

    // Key question: does color_idx=0 mean transparent in 4bpp mode?
    // YES - in GBA, palette index 0 is always transparent for BG/OBJ tiles
    // So if all layers have color_idx=0, the backdrop color (from DISPCNT) is used

    // Check backdrop color (palette entry 0,0)
    let palette = gba.mem.palette();
    let backdrop = u16::from_le_bytes([palette[0], palette[1]]);
    let br = (backdrop & 0x1F) as u32;
    let bg = ((backdrop >> 5) & 0x1F) as u32;
    let bb = ((backdrop >> 10) & 0x1F) as u32;
    println!(
        "\nBackdrop color: 0x{:04X} -> RGB({},{},{})",
        backdrop, br, bg, bb
    );
    println!(
        "  Expected FB: 0x{:02X}{:02X}{:02X}",
        br * 8 + br / 4,
        bg * 8 + bg / 4,
        bb * 8 + bb / 4
    );

    // Check what pixel (0,0) is in FB
    println!("\nFB at (0,0): 0x{:06X}", fb[0] & 0xFFFFFF);
    println!("FB at (120,0): 0x{:06X}", fb[120] & 0xFFFFFF);
    println!("FB at (239,0): 0x{:06X}", fb[239] & 0xFFFFFF);
}

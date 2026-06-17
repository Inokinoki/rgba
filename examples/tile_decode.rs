use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];
    let mut frame = 0u32;

    for _ in 0..240 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }
    gba.input.press_key(KeyState::START);
    for _ in 0..60 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }
    gba.input.release_key(KeyState::START);
    for _ in 0..120 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }

    for round in 0..100 {
        gba.input.press_key(KeyState::A);
        for _ in 0..3 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }
        gba.input.release_key(KeyState::A);
        for _ in 0..12 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }
    }

    let vram = gba.mem().vram();
    
    // Dump tile 279 raw data
    let tile_279_offset = 279 * 32;
    println!("Tile 279 at VRAM offset {:#06X}:", tile_279_offset);
    for row in 0..8 {
        let off = tile_279_offset + row * 4;
        let bytes = &vram[off..off+4];
        let mut pixels = String::new();
        for byte in bytes {
            let low = byte & 0x0F;
            let high = (byte >> 4) & 0x0F;
            pixels.push_str(&format!("{}{}", low, high));
        }
        println!("  Row {}: {:02X} {:02X} {:02X} {:02X}  pixels: {}", row, bytes[0], bytes[1], bytes[2], bytes[3], pixels);
    }

    // Dump tiles 276-284
    println!("\nTile data for BG3 tiles (276-284):");
    for tile in 276..=284 {
        let off = tile * 32;
        let nonzero: usize = vram[off..off+32].iter().map(|&b| if b != 0 { 1 } else { 0 }).sum();
        if nonzero > 0 {
            println!("  Tile {} at {:#06X}: {} non-zero bytes", tile, off, nonzero);
            // First row
            println!("    First row: {:02X} {:02X} {:02X} {:02X}", vram[off], vram[off+1], vram[off+2], vram[off+3]);
        } else {
            println!("  Tile {} at {:#06X}: EMPTY", tile, off);
        }
    }

    // Dump palette 0 (BG palette)
    println!("\nBG Palette 0 (first 16 colors):");
    for i in 0..16 {
        let color = gba.mem().read_palette_color(0, i);
        let r = color & 0x1F;
        let g = (color >> 5) & 0x1F;
        let b = (color >> 10) & 0x1F;
        if color != 0 {
            println!("  Color {}: {:#06X} = RGB({},{},{})", i, color, r*8, g*8, b*8);
        }
    }

    // Also check BG3's screen entries more completely
    println!("\nBG3 screen entries (rows 0-30, first 5 cols + cols 15-17):");
    let scr_base = 0xE000;
    for row in 0..30 {
        let mut line = format!("Row {:2}:", row);
        for col in [0u16, 1, 2, 15, 16, 17, 30, 31].iter() {
            let off = scr_base + (row * 32 + *col as usize) * 2;
            let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
            let tile = entry & 0x3FF;
            let pal = (entry >> 12) & 0xF;
            line.push_str(&format!(" [{},p{}]", tile, pal));
        }
        println!("{}", line);
    }
}

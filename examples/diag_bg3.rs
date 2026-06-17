use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let mut gba = Gba::new();
    gba.load_rom_path(rom_path).unwrap();

    for _ in 0..500 {
        gba.run_frame();
    }

    gba.sync_ppu_full();
    gba.sync_ppu();

    let ppu = gba.ppu();
    let vram = ppu.vram();

    // BG3: screen_base=0xF000, char_base=0x0000, 4bpp, size=0 (256x256)
    // Check first few screen entries of BG3
    println!("=== BG3 screen entries at 0xF000 ===");
    for row in 0..4 {
        for col in 0..8 {
            let off = 0xF000 + (row * 32 + col) * 2;
            let e = u16::from_le_bytes([vram[off], vram[off + 1]]);
            let tile_num = e & 0x3FF;
            let pal = (e >> 12) & 0xF;
            print!(" [{:3}/p{}]", tile_num, pal);
        }
        println!();
    }

    // Check tile 0 data (4bpp, 32 bytes)
    println!("\n=== Tile 0 (4bpp) at char_base=0 ===");
    for row in 0..2 {
        let off = row * 4;
        print!("Row {}: ", row);
        for byte in 0..4 {
            let b = vram[off + byte];
            print!("{:02X} ", b);
        }
        println!();
    }

    // Check what palette 0 colors look like
    println!("\n=== First 16 palette entries ===");
    let pal = gba.mem().palette();
    for i in 0..16 {
        let c = u16::from_le_bytes([pal[i * 2], pal[i * 2 + 1]]);
        let r = c & 0x1F;
        let g = (c >> 5) & 0x1F;
        let b = (c >> 10) & 0x1F;
        print!("[{:2}: rgb({:2},{:2},{:2})] ", i, r, g, b);
        if i % 4 == 3 {
            println!();
        }
    }

    // The key question: BG3 priority=0, so it renders ON TOP of everything
    // If BG3 is all white, it hides all other layers
    // Let's check: what if we render WITHOUT BG3?
    println!("\n=== Rendering with only BG0-2 ===");
    let dc = ppu.get_dispcnt();
    println!("Current DISPCNT={:#06X} (BG0-3+OBJ enabled)", dc);

    // What color would a BG3 pixel at (120,80) be?
    if let Some(c) = gba.get_bg_pixel(ppu, 0, 3, 120, 80) {
        println!("BG3 pixel (120,80): {:#06X}", c);
    }

    // Check BG2 at same location
    if let Some(c) = gba.get_bg_pixel(ppu, 0, 2, 120, 80) {
        println!("BG2 pixel (120,80): {:#06X}", c);
    }

    // Check what tile data is at tile 0 with different palettes
    println!("\n=== Checking if tile data is actually nonzero ===");
    for tile in [0, 1, 2, 100, 319] {
        let off = tile * 32;
        let mut nonzero = 0;
        for b in &vram[off..off + 32] {
            if *b != 0 {
                nonzero += 1;
            }
        }
        print!("Tile {}: {} nonzero bytes | ", tile, nonzero);
        // decode first row
        for byte in 0..4 {
            let b = vram[off + byte];
            print!("{:02X} ", b);
        }
        println!();
    }
}

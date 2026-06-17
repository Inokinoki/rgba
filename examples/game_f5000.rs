use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for _ in 0..5000 {
        gba.run_frame();
    }

    let vram = gba.mem().vram();

    // Check tile 394
    let tile_off = 394 * 32;
    let mut sum = 0u32;
    for i in 0..32 {
        sum += vram[tile_off + i] as u32;
    }
    println!("Frame 5000: tile 394 sum={}", sum);

    // Check tiles 187-200
    for tile in [187, 200, 256, 300, 394, 500, 600, 611] {
        let off = tile * 32;
        let mut s = 0u32;
        for i in 0..32 {
            s += vram[off + i] as u32;
        }
        if s > 0 {
            print!("Tile {}={}", tile, s);
            // Show first 8 bytes
            print!(" [");
            for i in 0..8 {
                print!("{:02X}", vram[off + i]);
            }
            print!("]");
        }
        println!();
    }

    // Total BG tile data
    let mut total = 0u64;
    for i in 0..0x4000u32 {
        total += vram[i as usize] as u64;
    }
    println!("\nBG VRAM total: {}", total);

    // Max non-zero tile
    let mut max_tile = 0;
    for tile in 0..1024 {
        let off = tile * 32;
        let mut has = false;
        for i in 0..32 {
            if vram[off + i] != 0 {
                has = true;
                break;
            }
        }
        if has {
            max_tile = tile;
        }
    }
    println!("Max tile with data: {}", max_tile);

    // BG0 tilemap
    let io = gba.mem().io();
    let bg0cnt = u16::from_le_bytes([io[0x08], io[0x09]]);
    let sb = ((bg0cnt >> 8) & 0x1F) as usize * 0x800;
    println!("\nBG0 first row:");
    for i in 0..64 {
        let off = sb + i * 2;
        let e = u16::from_le_bytes([vram[off], vram[off + 1]]);
        let t = e & 0x3FF;
        if t != 0x3FF {
            print!("{:4}", t);
        } else {
            print!("   .");
        }
        if (i + 1) % 32 == 0 {
            println!();
        }
    }
}

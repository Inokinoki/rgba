use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..500 {
        gba.run_frame_parallel(&mut fb);
    }

    // Check palette
    let pal = gba.mem.palette();
    println!("BG Palette[0-15]:");
    for i in 0..16 {
        let c = u16::from_le_bytes([pal[i * 2], pal[i * 2 + 1]]);
        print!("  [{:2}]={:04X}", i, c);
        if i % 8 == 7 {
            println!();
        }
    }

    println!("\nOBJ Palette[0-15] (at 0x200):");
    for i in 0..16 {
        let off = 0x200 + i * 2;
        let c = u16::from_le_bytes([pal[off], pal[off + 1]]);
        print!("  [{:2}]={:04X}", i, c);
        if i % 8 == 7 {
            println!();
        }
    }

    println!("\nOBJ Palette[16-31]:");
    for i in 16..32 {
        let off = 0x200 + i * 2;
        let c = u16::from_le_bytes([pal[off], pal[off + 1]]);
        print!("  [{:2}]={:04X}", i, c);
        if (i - 16) % 8 == 7 {
            println!();
        }
    }

    // Count total non-zero entries
    let mut bg_nonzero = 0;
    let mut obj_nonzero = 0;
    for i in 0..256 {
        let c = u16::from_le_bytes([pal[i * 2], pal[i * 2 + 1]]);
        if c != 0 {
            bg_nonzero += 1;
        }
    }
    for i in 0..256 {
        let off = 0x200 + i * 2;
        let c = u16::from_le_bytes([pal[off], pal[off + 1]]);
        if c != 0 {
            obj_nonzero += 1;
        }
    }
    println!("\nBG palette: {} non-zero of 256", bg_nonzero);
    println!("OBJ palette: {} non-zero of 256", obj_nonzero);
}

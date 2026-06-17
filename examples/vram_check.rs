use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..500u32 {
        gba.run_frame_parallel(&mut fb);
    }

    let vram = gba.mem.vram();

    // Check OBJ VRAM area
    let mut nz_count = 0;
    for off in (0x10000..0x18000).step_by(4) {
        let v = u32::from_le_bytes([vram[off], vram[off + 1], vram[off + 2], vram[off + 3]]);
        if v != 0 {
            nz_count += 1;
        }
    }
    println!(
        "Non-zero words in OBJ VRAM: {}/{}",
        nz_count,
        (0x18000 - 0x10000) / 4
    );

    // Find first non-zero tile in OBJ area
    for tile in 0..512 {
        let off = 0x10000 + tile * 32;
        let mut has_data = false;
        for b in 0..32 {
            if vram[off + b] != 0 {
                has_data = true;
                break;
            }
        }
        if has_data {
            print!("First non-zero OBJ tile: {} at 0x{:X}: ", tile, off);
            for b in 0..8 {
                print!("{:02X} ", vram[off + b]);
            }
            println!();
            break;
        }
    }

    // Check tiles 768, 784, 842, 848
    for tile in [768, 784, 842, 848] {
        let off = 0x10000 + tile * 32;
        if off + 32 <= vram.len() {
            let mut nz = 0;
            for b in 0..32 {
                if vram[off + b] != 0 {
                    nz += 1;
                }
            }
            print!("Tile {}: {} non-zero bytes", tile, nz);
            if nz > 0 {
                print!(" data: ");
                for b in 0..8 {
                    print!("{:02X} ", vram[off + b]);
                }
            }
            println!();
        } else {
            println!(
                "Tile {}: offset 0x{:X} out of bounds (vram len {})",
                tile,
                off,
                vram.len()
            );
        }
    }

    // Framebuffer at sprite location
    let pixel = fb[116 * 240 + 100];
    println!("\nFramebuffer at (100,116): {:08X}", pixel);

    // Check what pixel value the rendered frame has near sprites
    for y in 116..132 {
        let row_start = y * 240;
        let mut has_color = false;
        for x in 100..116 {
            if fb[row_start + x] != 0 {
                has_color = true;
                break;
            }
        }
        if has_color {
            print!("Row {}: ", y);
            for x in 100..116 {
                print!("{:08X} ", fb[row_start + x]);
            }
            println!();
            break;
        }
    }
}

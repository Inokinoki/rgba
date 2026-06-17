use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/Builds/gba-tests/thumb/thumb.gba")
        .unwrap();

    for _ in 0..300 {
        gba.run_frame();
    }

    let vram = gba.ppu().vram();

    // Print first 240 bytes (first scanline) - check for text at position (56, 76)
    // Text position: m_text_pos 60, 76 for "Failed test" or 56, 76 for "All tests passed"
    // In mode 4: pixel at (x,y) = vram[y*240+x]

    println!("=== Scanline 76 (text y position) ===");
    let y = 76usize;
    for x in 50..190 {
        let b = vram[y * 240 + x];
        if b != 0 {
            print!("{:02X}", b);
        } else {
            print!("..");
        }
    }
    println!();

    // Print scanlines 76-91 (16 lines of text)
    println!("=== Scanlines 76-91 ===");
    for y in 76..92 {
        print!("Y{:03}: ", y);
        for x in 50..190 {
            let b = vram[y * 240 + x];
            if b != 0 {
                print!("#");
            } else {
                print!(" ");
            }
        }
        println!();
    }

    // Also check scanlines 76-91 for "All tests passed" at (56, 76)
    println!("\n=== Scanlines 76-91 at x=56 ===");
    for y in 76..92 {
        print!("Y{:03}: ", y);
        for x in 56..184 {
            let b = vram[y * 240 + x];
            if b != 0 {
                print!("{}", if b == 1 { "#" } else { "+" });
            } else {
                print!(" ");
            }
        }
        println!();
    }
}

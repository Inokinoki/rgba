use rgba::Gba;

fn main() {
    let tests = [
        ("arm", "/home/ubuntu/Builds/gba-tests/arm/arm.gba"),
        ("thumb", "/home/ubuntu/Builds/gba-tests/thumb/thumb.gba"),
    ];

    for (name, path) in &tests {
        if !std::path::Path::new(path).exists() {
            continue;
        }
        let mut gba = Gba::new();
        gba.load_rom_path(path).unwrap();

        for _ in 0..300 {
            gba.run_frame();
        }

        gba.sync_ppu_full();

        let dc = gba.ppu().get_dispcnt();
        println!(
            "{}: DC={:#06X} PC={:#010X}",
            name,
            dc,
            gba.cpu().get_instruction_pc()
        );

        let vram = gba.mem().vram();
        let mut nz = 0usize;
        for &b in vram.iter() {
            if b != 0 {
                nz += 1;
            }
        }
        println!("  VRAM non-zero: {}/{}", nz, vram.len());

        println!("  VRAM[0x0000..0x0040]:");
        for row in 0..4 {
            print!("    ");
            for col in 0..16 {
                print!("{:02X} ", vram[row * 16 + col]);
            }
            println!();
        }

        let palette = gba.mem().palette();
        println!("  Palette[0..8]:");
        for i in 0..8 {
            let color = u16::from_le_bytes([palette[i * 2], palette[i * 2 + 1]]);
            print!(" {:04X}", color);
        }
        println!();

        println!("  First 16 bytes of VRAM line 0 (y=0):");
        for x in 0..16 {
            print!("{:02X} ", vram[x]);
        }
        println!();

        let mut r0 = gba.cpu().get_reg(0);
        let mut r1 = gba.cpu().get_reg(1);
        let mut r2 = gba.cpu().get_reg(2);
        let mut r3 = gba.cpu().get_reg(3);
        println!(
            "  R0={:#010X} R1={:#010X} R2={:#010X} R3={:#010X}",
            r0, r1, r2, r3
        );

        let is_thumb = gba.cpu().is_thumb_mode();
        println!("  Thumb mode: {}", is_thumb);
    }
}

use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/Builds/gba-tests/arm/arm.gba")
        .unwrap();

    println!("=== Direct VRAM write test ===");
    gba.write_half(0x06000100, 0x1234);
    gba.write_half(0x06000102, 0xABCD);
    gba.write_word(0x06000200, 0xDEADBEEF);
    let vram = gba.mem().vram();
    println!(
        "write_half(0x100, 0x1234): [{:02X} {:02X}]",
        vram[0x100], vram[0x101]
    );
    println!(
        "write_half(0x102, 0xABCD): [{:02X} {:02X}]",
        vram[0x102], vram[0x103]
    );
    println!(
        "write_word(0x200, 0xDEADBEEF): [{:02X} {:02X} {:02X} {:02X}]",
        vram[0x200], vram[0x201], vram[0x202], vram[0x203]
    );

    println!("\n=== Run ARM test for 300 frames ===");
    gba.load_rom_path("/home/ubuntu/Builds/gba-tests/arm/arm.gba")
        .unwrap();
    for _ in 0..300 {
        gba.run_frame();
    }

    println!("PC={:#010X}", gba.cpu().get_instruction_pc());
    println!(
        "R0={:#010X} R1={:#010X} R12={:#010X}",
        gba.cpu().get_reg(0),
        gba.cpu().get_reg(1),
        gba.cpu().get_reg(12)
    );

    let vram = gba.mem().vram();
    let mut nz = 0usize;
    for &b in vram.iter() {
        if b != 0 {
            nz += 1;
        }
    }
    println!("VRAM non-zero: {}/{}", nz, vram.len());

    let palette = gba.mem().palette();
    let mut pnz = 0usize;
    for &b in palette.iter() {
        if b != 0 {
            pnz += 1;
        }
    }
    println!("Palette non-zero: {}/{}", pnz, palette.len());

    println!(
        "Palette[0..4]: {:02X} {:02X}  {:02X} {:02X}  {:02X} {:02X}  {:02X} {:02X}",
        palette[0],
        palette[1],
        palette[2],
        palette[3],
        palette[4],
        palette[5],
        palette[6],
        palette[7]
    );

    println!("VRAM[0xF000..0xF010]:");
    for i in 0..16 {
        print!("{:02X} ", vram[0xF000 + i]);
    }
    println!();

    let io = gba.mem().io();
    let dispcnt = u16::from_le_bytes([io[0], io[1]]);
    println!("DISPCNT={:#06X}", dispcnt);
}

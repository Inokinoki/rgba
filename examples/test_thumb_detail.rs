use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/Builds/gba-tests/thumb/thumb.gba").unwrap();

    // Run for 300 frames
    for _ in 0..300 {
        gba.run_frame();
    }

    println!("PC={:#010X}", gba.cpu().get_instruction_pc());
    println!("R12={:#010X}", gba.cpu().get_reg(12));
    println!("CPSR={:#010X}", gba.cpu().get_cpsr());
    
    // Check all registers
    for i in 0..16 {
        println!("R{:02}={:#010X}", i, gba.cpu().get_reg(i));
    }

    // Check palette
    let pal = gba.mem().palette();
    let mut nonzero = 0;
    for i in (0..512).step_by(2) {
        let c = u16::from_le_bytes([pal[i], pal[i+1]]);
        if c != 0 {
            nonzero += 1;
            if nonzero <= 10 {
                println!("Pal[{:#04X}]={:#06X}", i/2, c);
            }
        }
    }
    println!("Non-zero palette entries: {}", nonzero);

    // Check VRAM
    let vram = gba.ppu().vram();
    let mut nz = 0;
    for &b in vram.iter().take(0xA000) {
        if b != 0 { nz += 1; }
    }
    println!("VRAM[0..0xA000] non-zero: {}", nz);
}

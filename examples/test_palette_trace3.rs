use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.mem.palette_log_enabled = true;
    gba.load_rom_path("/home/ubuntu/Builds/gba-tests/arm/arm.gba")
        .unwrap();

    for step in 0..15 {
        let prev_pc = gba.cpu().get_instruction_pc();

        // Before stepping, print register state
        println!("Step {} PC={:#010X}: R0={:#010X} R1={:#010X} R2={:#010X} R3={:#010X} R12={:#010X} LR={:#010X} SP={:#010X}",
            step, prev_pc, 
            gba.cpu().get_reg(0), gba.cpu().get_reg(1), gba.cpu().get_reg(2),
            gba.cpu().get_reg(3), gba.cpu().get_reg(12), gba.cpu().get_reg(14),
            gba.cpu().get_reg(13));

        let prev_log_len = gba.mem.palette_write_log.len();
        gba.step();

        let log = &gba.mem.palette_write_log;
        if log.len() > prev_log_len {
            for i in prev_log_len..log.len() {
                let (addr, val) = log[i];
                println!("  >>> PALETTE WRITE: addr={:#010X} val={:#04X}", addr, val);
            }
        }

        let pal = gba.mem().palette();
        let p0 = u16::from_le_bytes([pal[0], pal[1]]);
        let p2 = u16::from_le_bytes([pal[2], pal[3]]);
        let p4 = u16::from_le_bytes([pal[4], pal[5]]);
        if p0 != 0 || p2 != 0 || p4 != 0 {
            println!("  Palette: [0]={:#06X} [1]={:#06X} [2]={:#06X}", p0, p2, p4);
        }
    }
}

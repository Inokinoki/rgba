use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.mem.palette_log_enabled = true;
    gba.load_rom_path("/home/ubuntu/Builds/gba-tests/arm/arm.gba")
        .unwrap();

    // Focus on steps 7-9 where text_init returns
    for step in 0..12 {
        let pc = gba.cpu().get_instruction_pc();

        // Read the opcode at PC
        let opcode = gba.mem.read_word(pc);

        // Read top of stack
        let sp = gba.cpu().get_reg(13);
        let stack_top0 = gba.mem.read_word(sp);
        let stack_top4 = gba.mem.read_word(sp.wrapping_add(4));
        let stack_top8 = gba.mem.read_word(sp.wrapping_add(8));
        let stack_top12 = gba.mem.read_word(sp.wrapping_add(12));

        println!("Step {} PC={:#010X} opcode={:#010X}", step, pc, opcode);
        println!(
            "  R0={:#010X} R1={:#010X} R2={:#010X} R12={:#010X} LR={:#010X} SP={:#010X}",
            gba.cpu().get_reg(0),
            gba.cpu().get_reg(1),
            gba.cpu().get_reg(2),
            gba.cpu().get_reg(12),
            gba.cpu().get_reg(14),
            sp
        );
        println!("  Stack: [{:#010X}]={:#010X} [{:#010X}]={:#010X} [{:#010X}]={:#010X} [{:#010X}]={:#010X}",
            sp, stack_top0, sp+4, stack_top4, sp+8, stack_top8, sp+12, stack_top12);

        gba.step();
    }
}

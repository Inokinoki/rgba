use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.mem.palette_log_enabled = true;
    gba.load_rom_path("/home/ubuntu/Builds/gba-tests/arm/arm.gba")
        .unwrap();

    // Run instruction by instruction and check palette after each step
    println!("=== Tracing palette writes from the start ===");

    for step in 0..200 {
        gba.step();

        let pal = gba.mem().palette();
        let p0 = u16::from_le_bytes([pal[0], pal[1]]);
        let p2 = u16::from_le_bytes([pal[2], pal[3]]);
        let p4 = u16::from_le_bytes([pal[4], pal[5]]);

        if p0 != 0 || p2 != 0 || p4 != 0 {
            let pc = gba.cpu().get_instruction_pc();
            println!(
                "Step {}: PC={:#010X} Pal[0]={:#06X} Pal[1]={:#06X} Pal[2]={:#06X}",
                step, pc, p0, p2, p4
            );
        }
    }

    let log = &gba.mem.palette_write_log;
    println!("\n=== Palette write log ({} entries) ===", log.len());
    for (i, &(addr, val)) in log.iter().enumerate() {
        println!("  [{}] write addr={:#010X} val={:#04X}", i, addr, val);
    }

    // Also check what CPU registers look like
    println!("\n=== CPU state after 200 steps ===");
    println!("PC={:#010X}", gba.cpu().get_instruction_pc());
    for i in 0..16 {
        println!("R{:02}={:#010X}", i, gba.cpu().get_reg(i));
    }

    let cpsr = gba.cpu().get_cpsr();
    println!(
        "CPSR={:#010X} mode={:#04X} IRQ={} Thumb={}",
        cpsr,
        cpsr & 0x1F,
        (cpsr >> 7) & 1,
        (cpsr >> 5) & 1
    );
}

use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.mem.palette_log_enabled = true;
    gba.load_rom_path("/home/ubuntu/Builds/gba-tests/arm/arm.gba")
        .unwrap();

    println!("=== Initial state ===");
    println!("PC={:#010X}", gba.cpu().get_instruction_pc());
    println!("CPSR={:#010X}", gba.cpu().get_cpsr());

    let dispcnt = gba.mem.read_half(0x04000000);
    println!("Initial DISPCNT={:#06X}", dispcnt);

    for step in 0..200 {
        let prev_pc = gba.cpu().get_instruction_pc();
        let prev_log_len = gba.mem.palette_write_log.len();
        let prev_pal: [u8; 6] = {
            let p = gba.mem().palette();
            [p[0], p[1], p[2], p[3], p[4], p[5]]
        };

        gba.step();

        let pc = gba.cpu().get_instruction_pc();
        let log = &gba.mem.palette_write_log;
        let pal = gba.mem().palette();

        let pal_changed = prev_pal != [pal[0], pal[1], pal[2], pal[3], pal[4], pal[5]];

        if log.len() > prev_log_len || pal_changed {
            println!("Step {}: PC {:#010X}→{:#010X}", step, prev_pc, pc);
            for i in prev_log_len..log.len() {
                let (addr, val) = log[i];
                println!("  PALETTE LOG: addr={:#010X} val={:#04X}", addr, val);
            }
            if pal_changed {
                println!(
                    "  Pal[0..6]: {:02X}{:02X} {:02X}{:02X} {:02X}{:02X}",
                    pal[0], pal[1], pal[2], pal[3], pal[4], pal[5]
                );
            }
        }
    }

    let dispcnt = gba.mem.read_half(0x04000000);
    println!(
        "\nAfter 200 steps: DISPCNT={:#06X} PC={:#010X}",
        dispcnt,
        gba.cpu().get_instruction_pc()
    );

    println!(
        "R0={:#010X} R1={:#010X} R2={:#010X} R12={:#010X}",
        gba.cpu().get_reg(0),
        gba.cpu().get_reg(1),
        gba.cpu().get_reg(2),
        gba.cpu().get_reg(12)
    );
}

use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/Builds/gba-tests/arm/arm.gba")
        .unwrap();

    println!("=== Test 1: Direct write_half to palette ===");
    gba.write_half(0x05000000, 0x7FFF);
    gba.write_half(0x05000002, 0x001F);
    gba.write_half(0x05000004, 0x03E0);
    let pal = gba.mem().palette();
    println!("After direct writes:");
    println!("  Pal[0..2] (expect 7FFF): {:02X}{:02X}", pal[0], pal[1]);
    println!("  Pal[2..4] (expect 001F): {:02X}{:02X}", pal[2], pal[3]);
    println!("  Pal[4..6] (expect 03E0): {:02X}{:02X}", pal[4], pal[5]);

    println!("\n=== Test 2: Read back palette ===");
    let v0 = gba.mem.read_half(0x05000000);
    let v1 = gba.mem.read_half(0x05000002);
    let v2 = gba.mem.read_half(0x05000004);
    println!("read_half(05000000)={:#06X} (expect 0x7FFF)", v0);
    println!("read_half(05000002)={:#06X} (expect 0x001F)", v1);
    println!("read_half(05000004)={:#06X} (expect 0x03E0)", v2);

    println!("\n=== Test 3: Reload and run ARM test for 300 frames ===");
    gba.mem.palette_log_enabled = true;
    gba.load_rom_path("/home/ubuntu/Builds/gba-tests/arm/arm.gba")
        .unwrap();
    for _ in 0..300 {
        gba.run_frame();
    }

    let pal = gba.mem().palette();
    let mut nonzero_colors = vec![];
    for i in (0..512).step_by(2) {
        let color = u16::from_le_bytes([pal[i], pal[i + 1]]);
        if color != 0 {
            nonzero_colors.push((i, color));
        }
    }
    println!("Non-zero palette entries: {}", nonzero_colors.len());
    for (off, color) in nonzero_colors.iter().take(20) {
        println!("  Pal[{:#06X}] = {:#06X}", off, color);
    }

    let log = &gba.mem.palette_write_log;
    println!("\nPalette write log entries: {}", log.len());
    for &(addr, val) in log.iter().take(30) {
        println!("  write addr={:#010X} val={:#04X}", addr, val);
    }

    if log.len() > 0 {
        let min_addr = log.iter().map(|&(a, _)| a).min().unwrap();
        let max_addr = log.iter().map(|&(a, _)| a).max().unwrap();
        println!("  Address range: {:#010X} - {:#010X}", min_addr, max_addr);
    }

    println!("\n=== CPU state ===");
    println!("PC={:#010X}", gba.cpu().get_instruction_pc());
    println!("R12={:#010X} (0=all pass)", gba.cpu().get_reg(12));

    let cpsr = gba.cpu().get_cpsr();
    println!(
        "CPSR={:#010X} mode={:#04X} IRQ={}",
        cpsr,
        cpsr & 0x1F,
        (cpsr >> 7) & 1
    );
}

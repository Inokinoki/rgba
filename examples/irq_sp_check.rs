use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..5 {
        gba.run_frame_parallel(&mut fb);
    }

    // After 5 frames, check IRQ SP and IWRAM content
    let cpu = gba.cpu();
    let sp_irq = cpu.banked_sp[1];
    println!("IRQ SP (banked_sp[1]): {:08X}", sp_irq);
    println!("Current mode: {:?}", cpu.get_mode());

    // Check if the game modified the IRQ SP
    // Read current SP value from IWRAM
    let mem = gba.mem();
    let iwram = &mem.iwram;

    // Check stack area
    println!("\nIWRAM around IRQ SP (0x03007F80-0x03007FFF):");
    for off in (0x7F80..0x7FA0).step_by(16) {
        print!("  {:04X}: ", off);
        for i in 0..16 {
            print!("{:02X} ", iwram[off + i]);
        }
        println!();
    }

    // Check if game overwrote the IRQ handler pointer at 0x03007FFC
    let handler_ptr =
        u32::from_le_bytes([iwram[0x7FFC], iwram[0x7FFD], iwram[0x7FFE], iwram[0x7FFF]]);
    println!("\nIRQ handler pointer at 0x03007FFC: {:08X}", handler_ptr);
}

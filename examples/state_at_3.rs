use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..3 { gba.run_frame_parallel(&mut fb); }

    // CPU state first
    println!("R0=0x{:08X} R1=0x{:08X} R2=0x{:08X} R3=0x{:08X}",
        gba.cpu().get_reg(0), gba.cpu().get_reg(1), gba.cpu().get_reg(2), gba.cpu().get_reg(3));
    println!("R4=0x{:08X} R5=0x{:08X} R6=0x{:08X} R7=0x{:08X}",
        gba.cpu().get_reg(4), gba.cpu().get_reg(5), gba.cpu().get_reg(6), gba.cpu().get_reg(7));
    println!("SP=0x{:08X} LR=0x{:08X} PC=0x{:08X}",
        gba.cpu().get_sp(), gba.cpu().get_lr(), gba.cpu().get_pc());
    let cpsr = gba.cpu().get_cpsr();
    println!("CPSR=0x{:08X}", cpsr);

    let m = gba.mem_mut();
    
    println!("\nIWRAM [0x03000000]:");
    for i in 0..16 {
        print!("{:08X} ", m.read_word(0x03000000 + i * 4));
        if (i + 1) % 8 == 0 { println!(); }
    }

    let ie = m.interrupt.ie.bits();
    let if_ = m.interrupt.if_raw.bits();
    let ime = m.interrupt.ime;
    println!("\nIE=0x{:04X} IF=0x{:04X} IME={}", ie, if_, ime);
    
    println!("\nhandler_table:");
    for i in 0..7 {
        print!("{:04X} ", m.read_half(0x03000430 + i * 2));
    }
    println!();
    
    println!("callback_table:");
    for i in 0..7 {
        print!("{:08X} ", m.read_word(0x03000450 + i * 4));
    }
    println!();
    
    println!("\n[0x03007FF8]={:08X} [0x03007FFC]={:08X}",
        m.read_word(0x03007FF8), m.read_word(0x03007FFC));
}

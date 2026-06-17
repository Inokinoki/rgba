use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); }

    let c = gba.cpu();
    println!("Our emulator regs at frame 10:");
    println!("R0=0x{:08X} R1=0x{:08X} R2=0x{:08X} R3=0x{:08X}",
        c.get_reg(0), c.get_reg(1), c.get_reg(2), c.get_reg(3));
    println!("R4=0x{:08X} R5=0x{:08X} R6=0x{:08X} R7=0x{:08X}",
        c.get_reg(4), c.get_reg(5), c.get_reg(6), c.get_reg(7));
    println!("R8=0x{:08X} R9=0x{:08X} R10=0x{:08X} R11=0x{:08X}",
        c.get_reg(8), c.get_reg(9), c.get_reg(10), c.get_reg(11));
    println!("R12=0x{:08X} SP=0x{:08X} LR=0x{:08X} PC=0x{:08X}",
        c.get_reg(12), c.get_sp(), c.get_lr(), c.get_pc());
    println!("CPSR=0x{:08X}", c.get_cpsr());
}

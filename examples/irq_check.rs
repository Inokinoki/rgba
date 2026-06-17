use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Use run_scanline and check after each one
    for sl in 0..228*5 {
        gba.run_scanline();
        
        // Check if the game IRQ handler at 0x03000958 was ever PC
        // by checking the last few PCs
        let mode = gba.cpu.get_mode();
        if mode == rgba::cpu::Mode::Irq {
            let pc = gba.cpu.get_pc();
            println!("Scanline {}: IRQ mode, PC={:08X}", sl, pc);
            if sl > 230 * 3 {
                break;
            }
        }
    }
}

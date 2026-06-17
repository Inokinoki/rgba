use rgba::Gba;
use std::collections::HashMap;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
    .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Run 3 frames to get past initial boot
    for _ in 0..3 {
        gba.run_frame_parallel(&mut fb);
    }

    // Now trace frame 4 in detail
    println!("Starting detailed trace...");
    let mut pc_log: Vec<(u32, u32)> = Vec::new(); // (scanline, pc)
    let mut irq_count = 0;

    for sl in 0..228 {
        let start_irq = gba.cpu().irq_save_count;
        gba.run_scanline();
        let end_irq = gba.cpu().irq_save_count;

        if end_irq != start_irq {
            irq_count += 1;
            let pc = gba.cpu().get_pc();
            let m = gba.mem_mut();
            let handler_ptr = m.read_word(0x03007FFC);
            let sp_irq = 0u32; // can't access private field
            let ie = m.interrupt.ie.bits();
            let if_ = m.interrupt.if_raw.bits();
            let ime = m.interrupt.ime;
            let in_int = m.interrupt.in_interrupt;
            let vblank_ctr = m.read_word(0x03007FF8);
            println!("IRQ at sl={} pc=0x{:08X} handler_ptr=0x{:08X} sp_irq=0x{:08X} IE=0x{:04X} IF=0x{:04X} IME={} in_int={} vblank_ctr={}",
                sl, pc, handler_ptr, sp_irq, ie, if_, ime, in_int, vblank_ctr);
        }
    }

    println!("\nTotal IRQs this frame: {}", irq_count);

    // Also dump BIOS region around 0x18
    let m = gba.mem_mut();
    print!("BIOS 0x18-0x4C: ");
    for i in (0x18usize..0x4Cusize).step_by(4) {
        let b = m.bios_read_word(i);
        print!("{:08X} ", b);
    }
    println!();

    // Dump handler code
    print!("Handler 0x03000958: ");
    for i in 0..8 {
        let addr = 0x03000958 + i * 4;
        print!("{:08X} ", m.read_word(addr));
    }
    println!();

    println!("vblank_ctr = {}", m.read_word(0x03007FF8));
    println!("[0x03007FFC] = 0x{:08X}", m.read_word(0x03007FFC));
    println!("[0x03007E00] = 0x{:08X}", m.read_word(0x03007E00));
}

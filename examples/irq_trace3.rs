use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    
    // Set VBLK to track
    {
        let iwram = gba.mem.iwram_mut();
        iwram[0x7FF8] = 0x42;
        iwram[0x7FF9] = 0x00;
        iwram[0x7FFA] = 0x00;
        iwram[0x7FFB] = 0x00;
    }
    
    // Enable CPU trace for just 2 frames
    gba.cpu.enable_trace();
    gba.run_frame_parallel(&mut fb);
    gba.run_frame_parallel(&mut fb);
    
    let trace = gba.cpu.get_trace();
    // Find entries where PC is at the IRQ handler or the BIOS IRQ vector
    let bios_irq = trace.iter().filter(|(pc, _, _, _)| *pc == 0x00000018).count();
    let game_handler = trace.iter().filter(|(pc, _, _, _)| *pc == 0x03000958 || (*pc >= 0x03000A1C && *pc <= 0x03000A48)).count();
    let swi_calls = trace.iter().filter(|(pc, opcode, _, _)| {
        // SWI in ARM: cond 1111 xxxxxxxx xxxxxxxx xxxxxxxx
        // SWI in THUMB: 11011111 xxxxxxxx
        (*opcode & 0x0F000000) == 0x0F000000 || (*opcode & 0xFF00) == 0xDF00
    }).count();
    
    println!("2 frames trace: {} entries", trace.len());
    println!("  BIOS IRQ vector (0x18): {} hits", bios_irq);
    println!("  Game IRQ handler (0x03000958): {} hits", game_handler);
    println!("  SWI-like instructions: {} hits", swi_calls);
    
    // Check VBLK
    println!("  VBLK after: {:08X}", gba.mem.read_word(0x03007FF8));
    
    // Find any entry where PC is in IWRAM (0x03000000-0x03007FFF)  
    let iwram_entries: Vec<_> = trace.iter().filter(|(pc, _, _, _)| *pc >= 0x03000000 && *pc < 0x03008000).collect();
    println!("  IWRAM PC entries: {}", iwram_entries.len());
    for (pc, opcode, regs, cpsr) in iwram_entries.iter().take(30) {
        println!("    PC={:08X} opcode={:08X} CPSR={:08X} R0={:08X}", pc, opcode, cpsr, regs[0]);
    }
}

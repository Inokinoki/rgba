use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    let mut fb = vec![0u32; 240 * 160];
    
    gba.cpu.trace_enabled = true;
    gba.cpu.trace_buf.clear();
    
    for frame in 0..3 {
        gba.run_frame_parallel(&mut fb);
    }
    
    // Find BIOS entries in trace
    let trace = &gba.cpu.trace_buf;
    println!("Trace entries: {}", trace.len());
    
    let bios_entries: Vec<_> = trace.iter().filter(|(pc, _, _, _)| *pc < 0x4000).collect();
    println!("BIOS entries: {}", bios_entries.len());
    
    for (pc, opcode, regs, cpsr) in bios_entries.iter().take(30) {
        let mode_bits = cpsr & 0x1F;
        let mode_name = match mode_bits {
            0x10 => "User",
            0x1F => "Sys",
            0x12 => "IRQ",
            0x13 => "SVC",
            _ => "???",
        };
        println!("[{:04X}] {:08X} {} R0={:08X} R1={:08X} R2={:08X} R3={:08X} SP={:08X} LR={:08X}", 
            pc, opcode, mode_name, regs[0], regs[1], regs[2], regs[3], regs[13], regs[14]);
    }
}

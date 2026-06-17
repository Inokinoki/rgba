use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    
    {
        let iwram = gba.mem.iwram_mut();
        iwram[0x7FF8] = 0x42;
        iwram[0x7FF9] = 0x00;
        iwram[0x7FFA] = 0x00;
        iwram[0x7FFB] = 0x00;
    }
    
    gba.cpu.enable_trace();
    gba.run_frame_parallel(&mut fb);
    gba.run_frame_parallel(&mut fb);
    
    let trace = gba.cpu.get_trace();
    
    // Print all IWRAM entries with register dumps
    for (pc, opcode, regs, cpsr) in trace.iter() {
        if *pc >= 0x03000A1C && *pc <= 0x03000A48 {
            println!("PC={:08X} op={:08X} R2={:08X} R3={:08X} R12={:08X} CPSR={:08X}",
                pc, opcode, regs[2], regs[3], regs[12], cpsr);
        }
    }
}

use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    
    // Check VBLK at sub-frame granularity
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    
    // Before frame 201
    let v1 = gba.mem.read_word(0x03007FF8);
    println!("Before frame 201: VBLK={:08X}", v1);
    
    gba.run_frame_parallel(&mut fb);
    let v2 = gba.mem.read_word(0x03007FF8);
    println!("After frame 201: VBLK={:08X} (delta={})", v2, v2 - v1);
    
    // Check what happens at the IRQ level
    // Enable IRQ trace
    gba.mem.irq_trace_enabled = true;
    gba.mem.irq_trace.clear();
    
    gba.run_frame_parallel(&mut fb);
    let v3 = gba.mem.read_word(0x03007FF8);
    println!("After frame 202: VBLK={:08X} (delta={})", v3, v3 - v2);
    
    // Check IRQ trace
    println!("\nIRQ trace ({} events):", gba.mem.irq_trace.len());
    for (pc, spsr, cpsr, if_reg, is_vblank) in gba.mem.irq_trace.iter().take(20) {
        println!("  PC={:08X} SPSR={:08X} CPSR={:08X} IF={:04X} VBlank={}", pc, spsr, cpsr, if_reg, is_vblank);
    }
}

use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    let mut fb = vec![0u32; 240 * 160];
    
    for _ in 0..240 { gba.run_frame_parallel(&mut fb); }
    
    println!("IRQ save count: {}", gba.cpu.irq_save_count);
    println!("IRQ restore count: {}", gba.cpu.irq_restore_count);
    println!("Save stack depth: {}", gba.cpu.irq_save_stack.len());
    
    let nonzero: usize = fb.iter().filter(|&&p| p != 0).count();
    println!("Framebuffer nonzero: {}", nonzero);
}

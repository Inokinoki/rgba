use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Run to frame 500
    for _ in 0..500 { gba.run_frame_parallel(&mut fb); }
    println!("Frame 500: DISPCNT={:04X}", gba.ppu().get_dispcnt());

    // Press START and trace one frame
    gba.input_mut().press_key(rgba::KeyState::START);
    
    // Run 10 frames
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
    gba.input_mut().release_key(rgba::KeyState::START);
    
    // Run 90 more frames
    for _ in 0..90 { gba.run_frame_parallel(&mut fb); }
    
    println!("Frame 600: DISPCNT={:04X}", gba.ppu().get_dispcnt());
    
    // Now press A and see what happens
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
    gba.input_mut().release_key(rgba::KeyState::A);
    
    // Check DISPCNT every 10 frames
    for i in 0..20 {
        gba.run_frame_parallel(&mut fb);
        let dispcnt = gba.ppu().get_dispcnt();
        let pc = gba.cpu_pc();
        println!("Frame {}: DISPCNT={:04X} PC={:08X}", 611 + i, dispcnt, pc);
        if dispcnt != 0x1F40 {
            println!("*** State changed! ***");
            break;
        }
    }
    
    // Also try: what if we just wait without pressing anything?
    let mut gba2 = Gba::new();
    gba2.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba2.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    let mut fb2 = vec![0u32; 240 * 160];
    
    for _ in 0..500 { gba2.run_frame_parallel(&mut fb2); }
    println!("\n--- No button presses, just wait ---");
    for i in 0..500 {
        gba2.run_frame_parallel(&mut fb2);
        let dispcnt = gba2.ppu().get_dispcnt();
        if dispcnt != 0x1F40 {
            println!("Frame {}: DISPCNT changed to {:04X}", 500 + i + 1, dispcnt);
            break;
        }
    }
    println!("Frame 1000: DISPCNT={:04X}", gba2.ppu().get_dispcnt());
}

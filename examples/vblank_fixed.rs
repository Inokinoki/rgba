use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    
    for frame in 0..=800 {
        gba.run_frame_parallel(&mut fb);
        if frame % 100 == 0 {
            let vblk = gba.mem.read_word(0x03007FF8);
            let dispcnt = gba.ppu().get_dispcnt();
            println!("Frame {:3}: VBLK={:08X} DISPCNT={:04X}", frame, vblk, dispcnt);
        }
    }
    
    // Now test with START+A
    println!("\n=== Testing START+A ===");
    let v1 = gba.mem.read_word(0x03007FF8);
    
    gba.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    println!("After START: DISPCNT={:04X} VBLK={:08X}", gba.ppu().get_dispcnt(), gba.mem.read_word(0x03007FF8));
    
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
    gba.input_mut().release_key(rgba::KeyState::A);
    for f in 0..300 {
        gba.run_frame_parallel(&mut fb);
        let dispcnt = gba.ppu().get_dispcnt();
        if f < 5 || f % 50 == 0 || dispcnt != 0x1F40 {
            println!("  Frame {}: DISPCNT={:04X}", 1010+f, dispcnt);
        }
    }
}

use rgba::Gba;
use std::fs;
use std::process::Command;

fn save_png(fb: &[u32], path: &str) {
    let ppm = path.replace(".png", ".ppm");
    let mut bytes = b"P6\n240 160\n255\n".to_vec();
    for y in 0..160 {
        for x in 0..240 {
            let p = fb[y * 240 + x];
            bytes.push(((p >> 16) & 0xFF) as u8);
            bytes.push(((p >> 8) & 0xFF) as u8);
            bytes.push((p & 0xFF) as u8);
        }
    }
    fs::write(&ppm, &bytes).unwrap();
    Command::new("python3")
        .args(["-c", &format!("from PIL import Image; Image.open('{}').save('{}')", ppm, path)])
        .output().unwrap();
}

fn main() {
    // Test 1: Trace with Flash64K, default 0xFF init
    println!("=== Test 1: Flash64K, 0xFF init ===");
    {
        let mut gba = Gba::new();
        gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
        gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
        gba.set_save_type(rgba::SaveType::Flash64K);
        
        // Log SRAM reads
        gba.mem.sram_read_log_enabled = true;
        
        let mut fb = vec![0u32; 240 * 160];
        
        // Run to title screen
        for i in 0..500 { gba.run_frame_parallel(&mut fb); }
        
        let sram_reads = &gba.mem.sram_read_log;
        println!("  SRAM reads during boot ({} total):", sram_reads.len());
        for (addr, val, frame) in sram_reads.iter().take(50) {
            println!("    [{:>4}] 0x{:08X} = 0x{:02X}", frame, addr, val);
        }
        if sram_reads.len() > 50 {
            println!("    ... and {} more reads", sram_reads.len() - 50);
        }
        
        // Check first 32 bytes of SRAM
        let sram_data: Vec<String> = (0..64).map(|i| {
            format!("{:02X}", gba.mem.read_byte(0x0E000000 + i))
        }).collect();
        println!("  SRAM first 64 bytes: {}", sram_data.join(" "));
        
        // Now press START+A and see what happens
        gba.mem.sram_read_log.clear();
        
        gba.input_mut().press_key(rgba::KeyState::START);
        for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
        gba.input_mut().release_key(rgba::KeyState::START);
        for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
        
        gba.input_mut().press_key(rgba::KeyState::A);
        for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
        gba.input_mut().release_key(rgba::KeyState::A);
        for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
        
        save_png(&fb, "/tmp/sram_trace_after_a.png");
        println!("\n  After START+A: DISPCNT={:04X}", gba.ppu().get_dispcnt());
        
        let sram_reads2 = &gba.mem.sram_read_log;
        println!("  SRAM reads during START+A ({} total):", sram_reads2.len());
        for (addr, val, frame) in sram_reads2.iter().take(50) {
            println!("    [{:>4}] 0x{:08X} = 0x{:02X}", frame, addr, val);
        }
    }
    
    // Test 2: Same but with SRAM initialized to 0x00
    println!("\n=== Test 2: Flash64K, 0x00 init ===");
    {
        let mut gba = Gba::new();
        gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
        gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
        gba.set_save_type(rgba::SaveType::Flash64K);
        // Manually zero out flash data
        gba.mem.zero_sram();
        
        let mut fb = vec![0u32; 240 * 160];
        
        for i in 0..500 { gba.run_frame_parallel(&mut fb); }
        println!("  Frame 500: DISPCNT={:04X}", gba.ppu().get_dispcnt());
        
        gba.input_mut().press_key(rgba::KeyState::START);
        for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
        gba.input_mut().release_key(rgba::KeyState::START);
        for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
        
        gba.input_mut().press_key(rgba::KeyState::A);
        for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
        gba.input_mut().release_key(rgba::KeyState::A);
        for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
        
        save_png(&fb, "/tmp/sram_zero_after_a.png");
        println!("  After START+A: DISPCNT={:04X}", gba.ppu().get_dispcnt());
    }
    
    // Test 3: SaveType::None (no save, SRAM returns 0xFF)
    println!("\n=== Test 3: SaveType::None ===");
    {
        let mut gba = Gba::new();
        gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
        gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
        // SaveType::None is default
        
        let mut fb = vec![0u32; 240 * 160];
        
        for i in 0..500 { gba.run_frame_parallel(&mut fb); }
        println!("  Frame 500: DISPCNT={:04X}", gba.ppu().get_dispcnt());
        
        gba.input_mut().press_key(rgba::KeyState::START);
        for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
        gba.input_mut().release_key(rgba::KeyState::START);
        for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
        
        gba.input_mut().press_key(rgba::KeyState::A);
        for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
        gba.input_mut().release_key(rgba::KeyState::A);
        for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
        
        save_png(&fb, "/tmp/sram_none_after_a.png");
        println!("  After START+A: DISPCNT={:04X}", gba.ppu().get_dispcnt());
    }
}

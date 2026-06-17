use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    
    // Run to the 2nd SWI 4 call
    // Frame 0: SWI 1, 11, 6
    // Frame 4: First SWI 4
    // Frame 6: Two SWI 4 calls (first and second)
    // So the 2nd SWI 4 is at frame 6
    // But let me check frame 7 - that's when the game is in the idle loop (3rd+ SWI 4)
    
    for frame in 0..7 { gba.run_frame_parallel(&mut fb); }
    
    // Now at frame 7 start - the game is in the idle loop
    // The CPU should be at 0x080D2F10 (just returned from SWI 4) or about to call SWI 4 again
    println!("CPU state after 7 frames:");
    println!("PC = 0x{:08X}", gba.cpu().get_pc());
    println!("R0 = 0x{:08X}", gba.cpu().get_reg(0));
    println!("R3 = 0x{:08X}", gba.cpu().get_reg(3));
    println!("SP = 0x{:08X}", gba.cpu().get_reg(13));
    println!("LR = 0x{:08X}", gba.cpu().get_reg(14));
    
    // Dump the same areas as mGBA
    println!("\n=== IWRAM 0x03000400-0x03000440 ===");
    for addr in (0x03000400..0x03000440u32).step_by(16) {
        let mut words = [0u32; 4];
        for i in 0..4 {
            words[i] = gba.mem_mut().read_word(addr + (i as u32)*4);
        }
        println!("0x{:08X}: {:08X} {:08X} {:08X} {:08X}", addr, words[0], words[1], words[2], words[3]);
    }
    
    println!("\n=== Stack area 0x03007E00-0x03007E40 ===");
    for addr in (0x03007E00..0x03007E40u32).step_by(16) {
        let mut words = [0u32; 4];
        for i in 0..4 {
            words[i] = gba.mem_mut().read_word(addr + (i as u32)*4);
        }
        println!("0x{:08X}: {:08X} {:08X} {:08X} {:08X}", addr, words[0], words[1], words[2], words[3]);
    }
    
    println!("\n=== EWRAM 0x02000C80-0x02000CC0 ===");
    for addr in (0x02000C80..0x02000CC0u32).step_by(16) {
        let mut words = [0u32; 4];
        for i in 0..4 {
            words[i] = gba.mem_mut().read_word(addr + (i as u32)*4);
        }
        println!("0x{:08X}: {:08X} {:08X} {:08X} {:08X}", addr, words[0], words[1], words[2], words[3]);
    }
    
    // Also dump the EWRAM data that the VBlank callback accesses
    // VBlank callback loads from IWRAM 0x03000410 → 0x02009208
    println!("\n=== EWRAM at 0x02009208 (VBlank callback target) ===");
    for addr in (0x02009208..0x02009248u32).step_by(16) {
        let mut words = [0u32; 4];
        for i in 0..4 {
            words[i] = gba.mem_mut().read_word(addr + (i as u32)*4);
        }
        println!("0x{:08X}: {:08X} {:08X} {:08X} {:08X}", addr, words[0], words[1], words[2], words[3]);
    }
    
    // Check what R5 points to (0x02009188) 
    println!("\n=== EWRAM at R5=0x02009188 ===");
    for addr in (0x02009188..0x020091C8u32).step_by(16) {
        let mut words = [0u32; 4];
        for i in 0..4 {
            words[i] = gba.mem_mut().read_word(addr + (i as u32)*4);
        }
        println!("0x{:08X}: {:08X} {:08X} {:08X} {:08X}", addr, words[0], words[1], words[2], words[3]);
    }
    
    // Dump IO registers
    println!("\n=== IO registers ===");
    let io = gba.mem().io();
    println!("DISPCNT  = 0x{:04X}", u16::from_le_bytes([io[0], io[1]]));
    println!("DISPSTAT= 0x{:04X}", u16::from_le_bytes([io[4], io[5]]));
    println!("VCOUNT   = 0x{:04X}", u16::from_le_bytes([io[6], io[7]]));
    println!("BG0CNT   = 0x{:04X}", u16::from_le_bytes([io[8], io[9]]));
    println!("BG1CNT   = 0x{:04X}", u16::from_le_bytes([io[0xA], io[0xB]]));
    println!("BG2CNT   = 0x{:04X}", u16::from_le_bytes([io[0xC], io[0xD]]));
    println!("BG3CNT   = 0x{:04X}", u16::from_le_bytes([io[0xE], io[0xF]]));
}

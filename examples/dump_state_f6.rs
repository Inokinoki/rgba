use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    
    // Run to end of frame 6 (when decompression finishes)
    for i in 0..7 { 
        gba.run_frame_parallel(&mut fb);
        println!("After frame {}: DISPCNT=0x{:04X}", i, {
            let io = gba.mem().io();
            u16::from_le_bytes([io[0], io[1]])
        });
    }

    // Dump EWRAM regions with data
    println!("\n=== EWRAM non-zero regions after frame 6 ===");
    let mut regions_found = 0;
    let mut addr = 0x0200_0000u32;
    while addr < 0x0204_0000u32 && regions_found < 20 {
        let val = gba.mem_mut().read_word(addr);
        if val != 0 {
            println!("0x{:08X}: 0x{:08X}", addr, val);
            regions_found += 1;
            // Skip ahead a bit
            addr += 4;
        } else {
            addr += 4;
        }
    }
    
    // Also check IWRAM for game state variables
    println!("\n=== IWRAM non-zero regions ===");
    let mut addr = 0x0300_0000u32;
    let mut count = 0;
    while addr < 0x0300_8000u32 && count < 50 {
        let val = gba.mem_mut().read_word(addr);
        if val != 0 {
            println!("0x{:08X}: 0x{:08X}", addr, val);
            count += 1;
        }
        addr += 4;
    }
    
    // Check game state variable (we know state=0 at this point)
    // The game uses 0x03004000+ area for game variables typically
    println!("\n=== Stack area (0x03007F00-0x03008000) ===");
    for addr in (0x03007F00..0x03008000u32).step_by(4) {
        let val = gba.mem_mut().read_word(addr);
        if val != 0 {
            println!("0x{:08X}: 0x{:08X}", addr, val);
        }
    }
    
    // Check what the main game state variables look like
    // I know from earlier analysis that state goes 0->1
    // Let me check 0x03004000 area
    println!("\n=== Game variables at 0x03004000+ ===");
    for addr in (0x03004000..0x03004100u32).step_by(4) {
        let val = gba.mem_mut().read_word(addr);
        if val != 0 {
            println!("0x{:08X}: 0x{:08X}", addr, val);
        }
    }
}

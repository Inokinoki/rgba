use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    
    // Run 7 frames (to the stuck state)
    for _ in 0..7 { gba.run_frame_parallel(&mut fb); }
    
    println!("Before patch: DISPCNT=0x{:04X}", {
        let io = gba.mem().io();
        u16::from_le_bytes([io[0], io[1]])
    });
    
    // Patch "Smsh" structure to match mGBA values
    let v04_mgba = 0x0B0A0004u32;
    let v08_mgba = 0x07040004u32;
    gba.mem_mut().write_word(0x02000C84, v04_mgba);
    gba.mem_mut().write_word(0x02000C88, v08_mgba);
    
    println!("Patched EWRAM+0x04 to 0x{:08X}, +0x08 to 0x{:08X}", v04_mgba, v08_mgba);
    
    // Run more frames and check if DISPCNT changes
    for frame in 0..30 {
        gba.run_frame_parallel(&mut fb);
        let dispcnt = {
            let io = gba.mem().io();
            u16::from_le_bytes([io[0], io[1]])
        };
        if dispcnt != 0x0080 || frame < 5 {
            println!("Frame {}: DISPCNT=0x{:04X}", 7 + frame + 1, dispcnt);
        }
        if dispcnt != 0x0080 && dispcnt != 0x0000 {
            println!("*** DISPCNT CHANGED! ***");
            // Dump more info
            let v04 = gba.mem_mut().read_word(0x02000C84);
            let v08 = gba.mem_mut().read_word(0x02000C88);
            println!("Smsh+0x04 = 0x{:08X}, +0x08 = 0x{:08X}", v04, v08);
            break;
        }
    }
    
    let dispcnt = {
        let io = gba.mem().io();
        u16::from_le_bytes([io[0], io[1]])
    };
    if dispcnt == 0x0080 {
        println!("Still stuck at DISPCNT=0x0080 after patch");
    }
}

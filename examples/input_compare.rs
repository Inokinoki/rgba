use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Run to frame 350 (title screen should be showing)
    for _ in 0..350 {
        gba.run_frame_parallel(&mut fb);
    }

    // Dump a wider range of memory
    println!("=== EWRAM 0x02000000-0x02000200 at frame 350 ===");
    for addr in (0x02000000..0x02000200).step_by(4) {
        let val = gba.mem.read_word(addr);
        if val != 0 {
            println!("{:08X}: {:08X}", addr, val);
        }
    }
    
    println!("\n=== IWRAM 0x03007E00-0x03008000 at frame 350 ===");
    for addr in (0x03007E00..0x03008000).step_by(4) {
        let val = gba.mem.read_word(addr);
        println!("{:08X}: {:08X}", addr, val);
    }
}

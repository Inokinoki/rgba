use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Count VBlank counter increments per frame
    // The counter at 0x03007FF8 increments each time SWI 0x05 runs
    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    println!("=== VBlank counter per frame (should be 1) ===");
    for f in 0..20 {
        let vbefore = gba.mem.read_word(0x03007FF8);
        gba.run_frame_parallel(&mut fb);
        let vafter = gba.mem.read_word(0x03007FF8);
        let diff = vafter.wrapping_sub(vbefore);
        println!(
            "Frame {}: counter {:08X} -> {:08X}  delta={}",
            200 + f,
            vbefore,
            vafter,
            diff
        );
    }

    // Also count SWI calls using the swi log
    println!("\n=== SWI call count per frame ===");
    gba.mem.swi_log_enabled = true;
    gba.mem.swi_log.clear();

    for f in 0..5 {
        gba.mem.swi_log.clear();
        gba.run_frame_parallel(&mut fb);
        let count = gba.mem.swi_log.len();
        let swi_nums: Vec<u32> = gba.mem.swi_log.iter().take(20).cloned().collect();
        println!(
            "Frame {}: {} SWIs {:?}",
            220 + f,
            count,
            &swi_nums[..std::cmp::min(swi_nums.len(), 10)]
        );
    }
}

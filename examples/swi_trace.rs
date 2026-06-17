use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    gba.mem_mut().swi_log_enabled = true;

    for frame in 0..6 {
        gba.run_frame_parallel(&mut fb);
    }

    let log = &gba.mem().swi_log;
    println!("SWI calls ({} total):", log.len());
    for (i, &swi) in log.iter().enumerate() {
        if swi == 0x04 {
            // We can't get registers at SWI call time easily
            // But we know the SWI is called
        }
        println!("{:4}: SWI 0x{:02X}", i, swi);
    }

    // The real question: does the game check IF directly?
    // On real GBA, IntrWait BIOS code:
    // 1. Checks if requested interrupt already pending
    // 2. If yes, sets [R1] = IF & IE, clears IF, returns
    // 3. If no, enables interrupts, halts
    // 4. On wake, sets [R1] = IF & IE, clears IF, returns

    // Our implementation just halts. When VBlank fires:
    // - CPU wakes from halt
    // - IRQ handler at 0x03000958 runs
    // - Handler processes interrupt flags
    // - CPU returns from IRQ
    // - But SWI 0x04 hasn't set [R1] or cleared IF
    // - So game loops back to IntrWait

    // The fix: SWI 0x04 needs to write [R1] = IF & IE and clear IF
    // But we don't know R1 in our stub implementation!
    // We need to actually implement the BIOS IntrWait behavior

    println!("\nTo fix: need to implement proper IntrWait in SWI handler");
    println!("The BIOS IntrWait code should:");
    println!("  1. Check R0 (0=wait for new, 1=wait for old)");
    println!("  2. Set [R1] = (IF & IE), clear IF bits");
    println!("  3. If no pending interrupt, halt");
    println!("  4. On wake, set [R1] = (IF & IE), clear IF bits");
}

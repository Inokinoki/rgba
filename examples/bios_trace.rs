use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    let mut bios_pcs_seen = std::collections::HashSet::new();
    let mut game_handler_called = false;
    let mut in_irq_mode_count = 0u32;

    // Run just a few frames with careful PC tracking
    for frame in 0..5 {
        // Run step by step for one frame
        for _ in 0..280896 {
            let pc = gba.cpu.get_pc();
            let mode = gba.cpu.get_mode();
            
            if mode == rgba::cpu::Mode::Irq {
                in_irq_mode_count += 1;
                if pc < 0x4000 && !bios_pcs_seen.contains(&pc) {
                    bios_pcs_seen.insert(pc);
                    println!("BIOS PC in IRQ: {:08X}", pc);
                }
                if pc >= 0x03000958 && pc < 0x03000A00 {
                    if !game_handler_called {
                        game_handler_called = true;
                        println!("GAME IRQ handler called! PC={:08X}", pc);
                    }
                }
            }
            
            gba.step();
        }
    }

    println!("\n=== Summary ===");
    println!("IRQ mode entries: {}", in_irq_mode_count);
    println!("Game handler called: {}", game_handler_called);
    println!("BIOS PCs seen in IRQ mode: {:?}", bios_pcs_seen.iter().collect::<Vec<_>>());
}

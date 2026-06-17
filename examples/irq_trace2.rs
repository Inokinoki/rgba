use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Track PC values during execution to see if IRQ handler runs
    let mut irq_handler_called = false;
    let mut pc_at_irq_handler = 0u32;
    
    for frame in 0..260 {
        // Before each frame, scan a few PCs
        gba.run_frame_parallel(&mut fb);
        
        if frame == 255 {
            // Press START+A
            gba.input.press_key(KeyState::START);
            gba.input.press_key(KeyState::A);
        }
    }

    // Run one more frame with tracing
    // Run scanline by scanline and check if PC ever hits IRQ handler
    for sl in 0..228 {
        gba.run_scanline();
        let pc = gba.cpu.get_pc();
        if pc >= 0x03000958 && pc < 0x03000A00 {
            if !irq_handler_called {
                irq_handler_called = true;
                pc_at_irq_handler = pc;
                println!("IRQ handler called! PC={:08X} at scanline {}", pc, sl);
            }
        }
    }

    if !irq_handler_called {
        println!("IRQ handler at 0x03000958 was NEVER called!");
    }
    
    // Check current state
    println!("State: {:08X}", gba.mem.read_word(0x02000074));
}

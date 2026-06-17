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
        .args([
            "-c",
            &format!(
                "from PIL import Image; Image.open('{}').save('{}')",
                ppm, path
            ),
        ])
        .output()
        .unwrap();
}

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Run to title screen
    for _ in 0..500 {
        gba.run_frame_parallel(&mut fb);
    }
    println!("Frame 500: DISPCNT={:04X}", gba.ppu().get_dispcnt());

    // Press START and release
    gba.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..10 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..30 {
        gba.run_frame_parallel(&mut fb);
    }

    // Now take snapshot of state before pressing A
    println!("\nBefore pressing A:");
    println!("  DISPCNT={:04X}", gba.ppu().get_dispcnt());
    let cpu = &gba.cpu;
    println!(
        "  PC={:08X} CPSR={:08X} mode={:?}",
        cpu.get_pc(),
        cpu.get_cpsr(),
        cpu.get_mode()
    );
    let regs = cpu.registers();
    println!(
        "  R0={:08X} R1={:08X} R2={:08X} R3={:08X}",
        regs[0], regs[1], regs[2], regs[3]
    );
    println!("  SP={:08X} LR={:08X}", regs[13], regs[14]);

    // Now press A and trace frame by frame
    println!("\n=== After pressing A ===");
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::A);

    // Check what happens each frame for a while
    for i in 0..60 {
        gba.run_frame_parallel(&mut fb);
        let dispcnt = gba.ppu().get_dispcnt();
        let pc = gba.cpu.get_pc();
        if i < 5 || i % 10 == 0 || dispcnt != 0x1F40 {
            println!("  Frame {}: DISPCNT={:04X} PC={:08X}", 541 + i, dispcnt, pc);
        }
        if dispcnt == 0x0080 || dispcnt == 0x1640 {
            save_png(&fb, &format!("/tmp/game_flow_f{}.png", 541 + i));
        }
    }

    // Check IWRAM state - look for game state variables
    println!("\n=== IWRAM State ===");
    let iwram = gba.mem.iwram();
    // Check common game state areas
    for offset in [0x7F00, 0x7E00, 0x7D00, 0x7C00, 0x7B00, 0x7A00] {
        let bytes: Vec<String> = (0..16)
            .map(|i| format!("{:02X}", iwram[offset + i]))
            .collect();
        println!("  IWRAM[0x{:04X}]: {}", offset, bytes.join(" "));
    }

    // Check 0x03007FF8 (VBlank counter) and 0x03007FFC (IRQ handler)
    let vblank_ctr =
        u32::from_le_bytes([iwram[0x7FF8], iwram[0x7FF9], iwram[0x7FFA], iwram[0x7FFB]]);
    let irq_handler =
        u32::from_le_bytes([iwram[0x7FFC], iwram[0x7FFD], iwram[0x7FFE], iwram[0x7FFF]]);
    println!("  VBlank counter: {:08X}", vblank_ctr);
    println!("  IRQ handler: {:08X}", irq_handler);

    // Also check what the game writes to EWRAM
    println!("\n=== EWRAM Key Areas ===");
    // The game likely stores its main state in EWRAM
    for addr in [
        0x02000000u32,
        0x02000010,
        0x02000020,
        0x02000030,
        0x02000100,
        0x02000200,
    ] {
        let val = gba.mem.read_word(addr);
        println!("  [{:08X}] = {:08X}", addr, val);
    }
}

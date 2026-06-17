use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let rom_data = std::fs::read(rom_path).unwrap();

    let frames = 200u32;
    let mut gba = Gba::new();
    gba.load_rom(rom_data.clone());
    for _ in 0..frames {
        gba.run_frame();
    }

    let io = gba.mem().io();

    // Interrupt state
    let ie = u16::from_le_bytes([io[0x200], io[0x201]]);
    let imE = u16::from_le_bytes([io[0x208], io[0x209]]);
    let if_ = u16::from_le_bytes([io[0x202], io[0x203]]);
    eprintln!("IE=0x{:04X} IME=0x{:04X} IF=0x{:04X}", ie, imE, if_);
    eprintln!(
        "VBlank IE: {} IME: {} IF: {}",
        ie & 1 != 0,
        imE & 1 != 0,
        if_ & 1 != 0
    );

    // CPU state
    let cpsr = gba.cpu_get_cpsr();
    eprintln!(
        "CPSR=0x{:08X} IRQ_disabled={} mode={}",
        cpsr,
        (cpsr >> 7) & 1,
        cpsr & 0x1F
    );
    eprintln!("PC=0x{:08X} LR=0x{:08X}", gba.cpu_pc(), gba.cpu_reg(14));

    // Halt state
    eprintln!("CPU halted: {}", gba.cpu().is_halted());

    // Check frame-by-frame progress - sample PC at various frames
    drop(io);

    let mut gba2 = Gba::new();
    gba2.load_rom(rom_data.clone());
    for i in 0..400u32 {
        gba2.run_frame();
        if i % 20 == 0 || i < 10 {
            let io2 = gba2.mem().io();
            let dispcnt = u16::from_le_bytes([io2[0], io2[1]]);
            let ie2 = u16::from_le_bytes([io2[0x200], io2[0x201]]);
            let ime2 = u16::from_le_bytes([io2[0x208], io2[0x209]]);
            let halted = gba2.cpu().is_halted();
            eprintln!(
                "F{:4}: PC=0x{:08X} DC=0x{:04X} IE=0x{:04X} IME=0x{:04X} halt={}",
                i,
                gba2.cpu_pc(),
                dispcnt,
                ie2,
                ime2,
                halted
            );
        }
    }

    // Now check what happens at VBlank transition during a frame
    // Run one more frame step by step and trace DMA3 activity
    eprintln!("\n=== Tracing DMA3 during frame 401 ===");
    let mut gba3 = Gba::new();
    gba3.load_rom(rom_data.clone());
    for _ in 0..400 {
        gba3.run_frame();
    }

    // Run 280896 steps for one frame, trace DMA3
    let mut vblank_count = 0u32;
    let mut dma3_fire_count = 0u32;
    let mut last_dma3_src = 0u32;
    let mut last_dma3_dst = 0u32;
    let mut last_dma3_cnt = 0u32;
    for step in 0..280896u32 {
        gba3.step();
        // Check DMA3 state after step
        let io3 = gba3.mem().io();
        let vcount = u16::from_le_bytes([io3[6], io3[7]]);
        let dispstat = u16::from_le_bytes([io3[4], io3[5]]);
        if dispstat & 1 != 0 && vblank_count == 0 {
            vblank_count = step;
            eprintln!("VBlank start at step {} (vcount={})", step, vcount);
        }
        // Check DMA3
        let dma3_base = 0xB0 + 3 * 12;
        let dma3_control = u16::from_le_bytes([io3[dma3_base + 10], io3[dma3_base + 11]]);
        let dma3_src = u32::from_le_bytes([
            io3[dma3_base],
            io3[dma3_base + 1],
            io3[dma3_base + 2],
            io3[dma3_base + 3],
        ]);
        let dma3_dst = u32::from_le_bytes([
            io3[dma3_base + 4],
            io3[dma3_base + 5],
            io3[dma3_base + 6],
            io3[dma3_base + 7],
        ]);
        let dma3_cnt = u16::from_le_bytes([io3[dma3_base + 8], io3[dma3_base + 9]]);
        if dma3_src != last_dma3_src
            || dma3_dst != last_dma3_dst
            || dma3_cnt != last_dma3_cnt as u16
            || dma3_control != last_dma3_cnt as u16
        {
            if dma3_fire_count < 5 {
                eprintln!(
                    "  Step {}: DMA3 src=0x{:08X} dst=0x{:08X} cnt={} ctrl=0x{:04X} vcount={}",
                    step, dma3_src, dma3_dst, dma3_cnt, dma3_control, vcount
                );
            }
            dma3_fire_count += 1;
            last_dma3_src = dma3_src;
            last_dma3_dst = dma3_dst;
            last_dma3_cnt = dma3_cnt as u32;
            last_dma3_cnt = dma3_control as u32;
        }
    }
    eprintln!("DMA3 changes during frame: {}", dma3_fire_count);
    eprintln!("VBlank at step: {}", vblank_count);
}

use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    gba.mem_mut().irq_trace_enabled = true;

    for frame in 0..5 {
        gba.run_frame_parallel(&mut fb);
        let iwram = gba.mem().iwram();
        let w0 = u32::from_le_bytes([iwram[0], iwram[1], iwram[2], iwram[3]]);
        let b8 = iwram[8];
        let pc = gba.cpu().get_pc();
        let ie = gba.mem().interrupt.ie.bits();
        let if_ = gba.mem().interrupt.if_raw.bits();
        println!(
            "F{}: [0000]=0x{:08X} [0008]=0x{:02X} PC=0x{:08X} IE=0x{:04X} IF=0x{:04X}",
            frame, w0, b8, pc, ie, if_
        );
    }

    let trace = &gba.mem().irq_trace;
    println!("\nIRQ events:");
    for (i, (kind, data, ie, if_, halted)) in trace.iter().enumerate() {
        let name = match kind {
            0 => "VBlank",
            1 => "Wake",
            2 => "TakeIRQ",
            _ => "?",
        };
        println!(
            "  {:4}: {} data=0x{:08X} IE=0x{:04X} IF=0x{:04X} halted={}",
            i, name, data, ie, if_, halted
        );
    }
}

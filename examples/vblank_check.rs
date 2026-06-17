use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    gba.mem_mut().irq_trace_enabled = true;

    for frame in 0..10 {
        gba.run_frame_parallel(&mut fb);
        let ie = gba.mem().interrupt.ie.bits();
        let if_ = gba.mem().interrupt.if_raw.bits();
        let ime = gba.mem().interrupt.ime;
        let in_int = gba.mem().interrupt.in_interrupt;
        let vblank_ctr = gba.mem_mut().read_word(0x03007FF8);
        let mode = gba.cpu().get_mode();
        println!(
            "F{:3}: IE=0x{:04X} IF=0x{:04X} IME={} in_int={} mode={:?} VBLK_ctr={}",
            frame, ie, if_, ime, in_int, mode, vblank_ctr
        );
    }

    let trace = &gba.mem().irq_trace;
    println!("\nAll events ({} total):", trace.len());
    for (i, (kind, data, ie, if_, halted)) in trace.iter().enumerate() {
        let name = match kind {
            0 => "VBlank",
            1 => "Wake",
            2 => "TakeIRQ",
            _ => "???",
        };
        println!(
            "  {:4}: {} data/pc=0x{:08X} IE=0x{:04X} IF=0x{:04X} halted={}",
            i, name, data, ie, if_, halted
        );
    }
}

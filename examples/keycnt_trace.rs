use rgba::Gba;
use rgba::KeyState;
use rgba::Interrupt;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..250 {
        gba.run_frame_parallel(&mut fb);
    }

    println!(">>> Pressing START+A at frame 250");
    gba.input.press_key(KeyState::START);
    gba.input.press_key(KeyState::A);

    // Enable irq trace
    gba.mem.irq_trace_enabled = true;
    gba.mem.irq_trace.clear();

    for frame in 250..260 {
        gba.run_frame_parallel(&mut fb);
        let io = gba.mem.io();
        let keyinput = u16::from_le_bytes([io[0x130], io[0x131]]);
        let keycnt = u16::from_le_bytes([io[0x132], io[0x133]]);
        println!(
            "Frame {:4}: KEYINPUT={:04X} KEYCNT={:04X} IE={:04X} IF={:04X} IME={}",
            frame, keyinput, keycnt,
            gba.mem.interrupt.ie.bits(),
            gba.mem.interrupt.if_raw.bits(),
            gba.mem.interrupt.ime,
        );
    }

    // Print IRQ trace
    println!("\n=== IRQ Trace ===");
    for (i, entry) in gba.mem.irq_trace.iter().enumerate() {
        println!("  {}: {:?}", i, entry);
    }
}

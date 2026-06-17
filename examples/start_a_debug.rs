use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..500 {
        gba.run_frame_parallel(&mut fb);
    }

    println!("=== Frame 500: Pressing START+A ===");
    gba.input.press_key(KeyState::START);
    gba.input.press_key(KeyState::A);

    for frame in 500..510 {
        gba.run_frame_parallel(&mut fb);
        let state = gba.mem.read_word(0x02000074);
        let io = gba.mem.io();
        let keyinput = u16::from_le_bytes([io[0x130], io[0x131]]);
        let dispcnt = u16::from_le_bytes([io[0], io[1]]);
        let buf1 = gba.mem.read_word(0x02000078);
        let buf2 = gba.mem.read_word(0x0200007C);
        println!(
            "Frame {:4}: STATE={:08X} DISPCNT={:04X} KEY={:04X} [78]={:08X} [7C]={:08X}",
            frame, state, dispcnt, keyinput, buf1, buf2
        );
    }

    gba.input.release_key(KeyState::START);
    gba.input.release_key(KeyState::A);

    println!("\n=== Released ===");
    for frame in 510..580 {
        gba.run_frame_parallel(&mut fb);
        if frame % 10 == 0 || frame == 568 || frame == 569 {
            let state = gba.mem.read_word(0x02000074);
            let io = gba.mem.io();
            let dispcnt = u16::from_le_bytes([io[0], io[1]]);
            println!("Frame {:4}: STATE={:08X} DISPCNT={:04X}", frame, state, dispcnt);
        }
    }
}

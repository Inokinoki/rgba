use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    gba.input_mut().press_key(KeyState::A);
    gba.input_mut().press_key(KeyState::START);

    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    // Check input struct layout at each candidate address
    let candidates = [
        0x02001BEC, 0x02001FC4, 0x02001FDE, 0x020089A0, 0x020089E4, 0x02008CF8, 0x0200929C,
        0x02009B76, 0x02009BB8, 0x02009BBA,
    ];

    for &base in &candidates {
        let w0 = gba.mem.read_half(base);
        let w4 = gba.mem.read_half(base + 4);
        let w8 = gba.mem.read_half(base + 8);
        let wc = gba.mem.read_half(base + 0xC);
        println!(
            "0x{:08X}: [0]={:04X} [4]={:04X} [8]={:04X} [C]={:04X}",
            base, w0, w4, w8, wc
        );
    }

    println!("\n=== Now test: release keys, run 1 frame, re-check ===");
    gba.input_mut().release_key(KeyState::A);
    gba.input_mut().release_key(KeyState::START);
    gba.run_frame_parallel(&mut fb);

    for &base in &candidates {
        let w0 = gba.mem.read_half(base);
        let w4 = gba.mem.read_half(base + 4);
        let w8 = gba.mem.read_half(base + 8);
        let wc = gba.mem.read_half(base + 0xC);
        println!(
            "0x{:08X}: [0]={:04X} [4]={:04X} [8]={:04X} [C]={:04X}",
            base, w0, w4, w8, wc
        );
    }

    println!("\n=== Now test: press only A, run 1 frame, re-check ===");
    gba.input_mut().press_key(KeyState::A);
    gba.run_frame_parallel(&mut fb);

    for &base in &candidates {
        let w0 = gba.mem.read_half(base);
        let w4 = gba.mem.read_half(base + 4);
        let w8 = gba.mem.read_half(base + 8);
        let wc = gba.mem.read_half(base + 0xC);
        println!(
            "0x{:08X}: [0]={:04X} [4]={:04X} [8]={:04X} [C]={:04X}",
            base, w0, w4, w8, wc
        );
    }
}

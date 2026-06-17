use rgba::Gba;
use rgba::KeyState;
fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];
    for _ in 0..240 { gba.run_frame_parallel(&mut framebuffer); }
    gba.input_mut().press_key(KeyState::START);
    for _ in 0..60 { gba.run_frame_parallel(&mut framebuffer); }
    gba.input_mut().release_key(KeyState::START);
    for _ in 0..120 { gba.run_frame_parallel(&mut framebuffer); }
    for _ in 0..100 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..3 { gba.run_frame_parallel(&mut framebuffer); }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..12 { gba.run_frame_parallel(&mut framebuffer); }
    }
    let vram = gba.mem().vram();
    // Check tile 1023 at offset 1023*32 = 32736 = 0x7FE0
    let off = 1023 * 32;
    println!("Tile 1023 at offset {:#X}:", off);
    let mut all_zero = true;
    for i in 0..32 {
        if vram[off + i] != 0 { all_zero = false; break; }
    }
    println!("  All zeros: {}", all_zero);
    if !all_zero {
        for row in 0..8 {
            print!("  row {}: ", row);
            for i in 0..4 { print!("{:02X} ", vram[off + row*4 + i]); }
            println!();
        }
    }
    
    // Check the last few tiles
    for t in 1020..1024 {
        let off = t * 32;
        let mut nz = 0;
        for i in 0..32 { if vram[off + i] != 0 { nz += 1; } }
        if nz > 0 { println!("Tile {}: {} non-zero bytes", t, nz); }
    }
}

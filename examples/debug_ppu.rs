use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    let mut fb = vec![0u32; 240 * 160];
    
    for _ in 0..240 { gba.run_frame_parallel(&mut fb); }
    
    // Read IO registers
    let dispcnt = gba.mem.read_half(0x04000000);
    let bg0cnt = gba.mem.read_half(0x04000008);
    let bg1cnt = gba.mem.read_half(0x0400000A);
    let bg2cnt = gba.mem.read_half(0x0400000C);
    let bg3cnt = gba.mem.read_half(0x0400000E);
    let ie = gba.mem.read_half(0x04000200);
    let ime = gba.mem.read_byte(0x04000208);
    
    println!("DISPCNT: 0x{:04X}", dispcnt);
    println!("BG0CNT: 0x{:04X}", bg0cnt);
    println!("BG1CNT: 0x{:04X}", bg1cnt);
    println!("BG2CNT: 0x{:04X}", bg2cnt);
    println!("BG3CNT: 0x{:04X}", bg3cnt);
    println!("IE: 0x{:04X}", ie);
    println!("IME: 0x{:02X}", ime);
    
    // Check some palette entries
    let pal0 = gba.mem.read_half(0x05000000);
    let pal1 = gba.mem.read_half(0x05000002);
    println!("Palette[0]: 0x{:04X}", pal0);
    println!("Palette[1]: 0x{:04X}", pal1);
    
    // Check DISPSTAT
    let dispstat = gba.mem.read_half(0x04000004);
    println!("DISPSTAT: 0x{:04X}", dispstat);
    
    // Check if any VRAM is non-zero
    let mut vram_nonzero = 0u32;
    for i in (0..0x18000).step_by(4) {
        let w = gba.mem.read_word(0x06000000 + i as u32);
        if w != 0 { vram_nonzero += 1; }
    }
    println!("VRAM non-zero words: {}/{}", vram_nonzero, 0x18000/4);
    
    // Frame buffer content
    let nonzero: usize = fb.iter().filter(|&&p| p != 0).count();
    println!("Framebuffer nonzero pixels: {}", nonzero);
}

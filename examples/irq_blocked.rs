use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    
    // Set VBLK to 0x42
    {
        let iwram = gba.mem.iwram_mut();
        iwram[0x7FF8] = 0x42;
        iwram[0x7FF9] = 0x00;
        iwram[0x7FFA] = 0x00;
        iwram[0x7FFB] = 0x00;
    }
    
    for frame in 0..5 {
        let before = gba.mem.read_word(0x03007FF8);
        let pc = gba.cpu.get_instruction_pc();
        let in_decomp = pc >= 0x080D0900 && pc < 0x080D0C20;
        gba.run_frame_parallel(&mut fb);
        let after = gba.mem.read_word(0x03007FF8);
        println!("Frame {}: VBLK {:08X}->{:08X} PC={:08X} in_decomp={}", frame, before, after, pc, in_decomp);
    }
}

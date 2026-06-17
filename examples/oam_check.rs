use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }

    // After run_frame_parallel, PPU state should be synced
    // But let's also check the memory system's OAM directly
    let mem_oam = gba.mem().oam();
    let ppu_oam = gba.ppu().oam();

    println!("Memory OAM first 64 bytes:");
    for i in (0..64).step_by(8) {
        print!("  {:03X}: ", i);
        for j in 0..8 {
            print!("{:02X} ", mem_oam[i + j]);
        }
        println!();
    }

    println!("\nPPU OAM first 64 bytes:");
    for i in (0..64).step_by(8) {
        print!("  {:03X}: ", i);
        for j in 0..8 {
            print!("{:02X} ", ppu_oam[i + j]);
        }
        println!();
    }

    // Parse sprites from memory OAM
    println!("\nSprites from memory OAM:");
    for s in 0..16 {
        let off = s * 8;
        let attr0 = u16::from_le_bytes([mem_oam[off], mem_oam[off + 1]]);
        let attr1 = u16::from_le_bytes([mem_oam[off + 2], mem_oam[off + 3]]);
        let attr2 = u16::from_le_bytes([mem_oam[off + 4], mem_oam[off + 5]]);
        let y = attr0 & 0xFF;
        let x = attr1 & 0x1FF;
        let tile = attr2 & 0x3FF;
        let obj_mode = (attr0 >> 10) & 3;
        if obj_mode == 2 {
            continue;
        }
        println!(
            "  Sprite {:2}: attr0={:04X} attr1={:04X} attr2={:04X}  x={} y={} tile={}",
            s, attr0, attr1, attr2, x, y, tile
        );
    }

    // Check how many non-zero bytes in OAM
    let nonzero_mem: usize = mem_oam.iter().filter(|&&b| b != 0).count();
    let nonzero_ppu: usize = ppu_oam.iter().filter(|&&b| b != 0).count();
    println!("\nMemory OAM nonzero: {}/{}", nonzero_mem, mem_oam.len());
    println!("PPU OAM nonzero: {}/{}", nonzero_ppu, ppu_oam.len());
}

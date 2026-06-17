use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();

    let mut tile_load_pcs: std::collections::HashSet<u32> = std::collections::HashSet::new();

    for frame in 0..10 {
        for _scanline in 0..228 {
            gba.run_scanline();
        }

        // Find PCs that wrote to BG tile area
        for &(addr, pc, _val) in &gba.mem().vram_write_log {
            if addr >= 0x06000000 && addr < 0x0600F000 {
                tile_load_pcs.insert(pc);
            }
        }

        println!("Frame {}:", frame);
        for &pc in &tile_load_pcs {
            println!("  PC={:08X}", pc);
        }
    }

    // Check what instruction is at each PC
    let rom = gba.mem().rom();
    for &pc in &tile_load_pcs {
        let offset = ((pc & !1) - 0x08000000) as usize;
        if offset + 2 <= rom.len() {
            let instr = u16::from_le_bytes([rom[offset], rom[offset + 1]]);
            println!("  {:08X}: {:04X} (THUMB)", pc, instr);
        }
    }
}

use rgba::Gba;

#[test]
fn trace_harvest_moon() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let rom_data = std::fs::read(rom_path).expect("Failed to read ROM");
    
    let mut gba = Gba::new();
    gba.load_rom(rom_data);
    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().dma_log_enabled = true;
    
    for frame in 0..5u32 {
        for _ in 0..280896 {
            gba.step();
        }
        
        let vram = gba.mem().vram();
        let mut ffff = 0u32;
        for i in 0..1024 {
            let off = 0xC000 + i * 2;
            let e = u16::from_le_bytes([vram[off], vram[off+1]]);
            if e == 0xFFFF { ffff += 1; }
        }
        let tff = (0..32).all(|i| vram[0x7FE0+i] == 0xFF);
        let t00 = (0..32).all(|i| vram[0x7FE0+i] == 0x00);
        eprintln!("Frame {}: 0xFFFF={} tile1023_ff={} tile1023_00={} PC=0x{:08X}", 
            frame, ffff, tff, t00, gba.cpu_pc());
    }
    
    let dma_log = &gba.mem().dma_log;
    eprintln!("\n=== DMA3 to VRAM ===");
    for &(ch, src, dst, count, size) in dma_log.iter() {
        if ch == 3 && dst >= 0x06000000 && dst < 0x06018000 {
            let smap = dst >= 0x0600C000 && dst < 0x0600C800;
            let tile = dst >= 0x06007FE0 && dst < 0x06008000;
            eprintln!("  DMA3: src=0x{:08X} dst=0x{:08X} count={} size={}B{}{}", 
                src, dst, count, size,
                if smap { " [SCREENMAP]" } else { "" },
                if tile { " [TILE1023]" } else { "" });
        }
    }
    
    let vlog = &gba.mem().vram_write_log;
    let smap_w: Vec<_> = vlog.iter().filter(|(a,_,_)| *a >= 0x0600C000 && *a < 0x0600C800).collect();
    let tile_w: Vec<_> = vlog.iter().filter(|(a,_,_)| *a >= 0x06007FE0 && *a < 0x06008000).collect();
    
    eprintln!("\nScreen map writes: {}", smap_w.len());
    for &(a,p,v) in smap_w.iter().take(20) {
        eprintln!("  0x{:08X}=0x{:02X} pc=0x{:08X}", a, v, p<<1);
    }
    
    eprintln!("\nTile 1023 writes: {}", tile_w.len());
    for &(a,p,v) in tile_w.iter().take(30) {
        eprintln!("  0x{:08X}=0x{:02X} pc=0x{:08X}", a, v, p<<1);
    }
}

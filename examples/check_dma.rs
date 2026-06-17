use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..7 { gba.run_frame_parallel(&mut fb); }
    
    let io = gba.mem().io();
    
    // DMA registers (0x040000B0-0x040000E0)
    // Each DMA channel: 4 words (source, dest, count, control)
    // DMA0: 0x040000B0, DMA1: 0x040000BC, DMA2: 0x040000C8, DMA3: 0x040000D4
    for ch in 0..4 {
        let base = 0xB0 + ch * 0xC;
        let src_lo = u16::from_le_bytes([io[base], io[base+1]]);
        let src_hi = u16::from_le_bytes([io[base+2], io[base+3]]);
        let src = ((src_hi as u32) << 16) | src_lo as u32;
        
        let dst_lo = u16::from_le_bytes([io[base+4], io[base+5]]);
        let dst_hi = u16::from_le_bytes([io[base+6], io[base+7]]);
        let dst = ((dst_hi as u32) << 16) | dst_lo as u32;
        
        let cnt_lo = u16::from_le_bytes([io[base+8], io[base+9]]);
        let cnt_hi = u16::from_le_bytes([io[base+10], io[base+11]]);
        
        println!("DMA{}: src=0x{:08X} dst=0x{:08X} count=0x{:04X} ctrl=0x{:04X}",
            ch, src, dst, cnt_lo, cnt_hi);
        
        if cnt_hi != 0 {
            println!("  enabled={} irq={} start={} bit26={}",
                cnt_hi & 0x8000 != 0,
                cnt_hi & 0x4000 != 0,
                match (cnt_hi >> 12) & 3 {
                    0 => "Immediate",
                    1 => "VBlank",
                    2 => "HBlank",
                    3 => "Special",
                    _ => "?"
                },
                (cnt_hi >> 10) & 1);
            let transfer_type = match (cnt_hi >> 10) & 1 {
                0 => "16bit",
                1 => "32bit",
                _ => "?"
            };
            println!("  type={} src_inc={} dst_inc={}",
                transfer_type,
                match (cnt_hi >> 7) & 3 {
                    0 => "Increment", 1 => "Decrement",
                    2 => "Fixed", 3 => "Reload",
                    _ => "?"
                },
                match (cnt_hi >> 5) & 3 {
                    0 => "Increment", 1 => "Decrement",
                    2 => "Fixed", 3 => "Reload",
                    _ => "?"
                });
        }
    }
    
    // Also dump the DMA struct state
    for i in 0..4 {
        let d = &gba.dma[i];
        println!("\nDMA{} struct: active={} enabled={}",
            i, d.is_active(), d.is_enabled());
    }
    
    // Check BG control registers
    println!("\n=== BG control registers ===");
    println!("BG0CNT = 0x{:04X}", u16::from_le_bytes([io[8], io[9]]));
    println!("BG1CNT = 0x{:04X}", u16::from_le_bytes([io[0xA], io[0xB]]));
    println!("BG2CNT = 0x{:04X}", u16::from_le_bytes([io[0xC], io[0xD]]));
    println!("BG3CNT = 0x{:04X}", u16::from_le_bytes([io[0xE], io[0xF]]));
    
    // Check for any IO writes happening via DMA
    // Look at what EWRAM data is near addresses that might be DMA sources
    println!("\n=== Potential DMA source data ===");
    for addr in [0x02008D2Cu32, 0x02008D1C, 0x02008D84, 0x0200918C] {
        let val = gba.mem_mut().read_word(addr);
        println!("EWRAM 0x{:08X} = 0x{:08X}", addr, val);
    }
}

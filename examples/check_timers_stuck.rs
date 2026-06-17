use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..8 { gba.run_frame_parallel(&mut fb); }

    let io = gba.mem().io();
    
    // Timer state
    for i in 0..4 {
        let base = 0x100 + i * 4;
        let cnt_l = u16::from_le_bytes([io[base], io[base+1]]);
        let cnt_h = u16::from_le_bytes([io[base+2], io[base+3]]);
        println!("TM{}: counter=0x{:04X} control=0x{:04X} (enable={} irq={} prescale={} cascade={})",
            i, cnt_l, cnt_h,
            cnt_h & 0x80 != 0,
            cnt_h & 0x40 != 0,
            cnt_h & 0x03,
            cnt_h & 0x04 != 0);
    }
    
    // Interrupt state
    let ie = gba.mem().interrupt.ie.bits();
    let if_ = gba.mem().interrupt.if_raw.bits();
    let ime = gba.mem().interrupt.ime;
    println!("\nIE=0x{:04X} IF=0x{:04X} IME={}", ie, if_, ime);
    println!("IE bits: VBlank={} HBlank={} VCount={} T0={} T1={} T2={} T3={}",
        ie & 1 != 0, ie & 2 != 0, ie & 4 != 0, ie & 8 != 0, ie & 16 != 0, ie & 32 != 0, ie & 64 != 0);
    
    // Check what mGBA timer state would be - compare with actual timer struct
    println!("\nTimer struct state:");
    for i in 0..4 {
        let t = &gba.timers[i];
        println!("TM{}: counter={} reload={} enabled={} irq={} count_up={}",
            i, t.get_counter(), t.get_reload(), t.is_enabled(), t.is_irq_enabled(), t.is_count_up());
    }
    
    // Disassemble the loop at 0x080D30CA
    println!("\n=== THUMB at 0x080D30C0-0x080D30E0 ===");
    for addr in (0x080D30C0..0x080D30E0).step_by(2) {
        let half = gba.mem_mut().read_half(addr);
        print!("0x{:08X}: 0x{:04X}  ", addr, half);
        // Simple decode
        if half >> 12 == 0xD && ((half >> 8) & 0xF) < 0xE {
            let off = (half & 0xFF) as i8 as i32;
            let tgt = (addr as i32 + 4 + off * 2) as u32;
            let cn = ["EQ","NE","CS","CC","MI","PL","VS","VC","HI","LS","GE","LT","GT","LE"];
            println!("B{} 0x{:08X}", cn[((half>>8)&0xF) as usize], tgt);
        } else if half >> 11 == 0b11100 {
            let off = (half & 0x7FF) as u32;
            let off = if off & 0x400 != 0 { (off|0xFFFFF800u32) as i32 } else { off as i32 };
            println!("B 0x{:08X}", (addr as i32+4+off*2) as u32);
        } else if half >> 11 == 0b01001 {
            println!("LDR R{},[PC,#{}]", (half>>8)&7, (half&0xFF)*4);
        } else if half >> 13 == 0b001 {
            let op = (half>>11)&3; let rd=(half>>8)&7; let imm=half&0xFF;
            println!("{} R{},#0x{:02X}", ["MOVS","CMP","ADDS","SUBS"][op as usize], rd, imm);
        } else {
            println!(".hword");
        }
    }
}

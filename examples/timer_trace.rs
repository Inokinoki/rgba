use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    gba.mem_mut().swi_log_enabled = true;

    for frame in 0..15 {
        gba.mem_mut().swi_log.clear();
        gba.run_frame_parallel(&mut fb);

        let swi_counts: std::collections::HashMap<u32, usize> = 
            gba.mem().swi_log.iter().fold(std::collections::HashMap::new(), |mut acc, &n| {
                *acc.entry(n).or_insert(0) += 1;
                acc
            });

        let io = gba.mem().io();
        let tm0_l = u16::from_le_bytes([io[0x100], io[0x101]]);
        let tm0_h = u16::from_le_bytes([io[0x102], io[0x103]]);
        let tm1_l = u16::from_le_bytes([io[0x104], io[0x105]]);
        let tm1_h = u16::from_le_bytes([io[0x106], io[0x107]]);
        let tm2_l = u16::from_le_bytes([io[0x108], io[0x109]]);
        let tm2_h = u16::from_le_bytes([io[0x10A], io[0x10B]]);
        let tm3_l = u16::from_le_bytes([io[0x10C], io[0x10D]]);
        let tm3_h = u16::from_le_bytes([io[0x10E], io[0x10F]]);
        let dispcnt = u16::from_le_bytes([io[0], io[1]]);
        let vcount = u16::from_le_bytes([io[6], io[7]]);
        
        let state = gba.mem_mut().read_word(0x02000074);
        let pc = gba.cpu().get_pc();
        
        println!("F{:3}: DISPCNT=0x{:04X} VCOUNT={} state={} PC=0x{:08X}",
            frame, dispcnt, vcount, state, pc);
        println!("  TM0: 0x{:04X}=0x{:04X}  TM1: 0x{:04X}=0x{:04X}",
            tm0_l, tm0_h, tm1_l, tm1_h);
        println!("  TM2: 0x{:04X}=0x{:04X}  TM3: 0x{:04X}=0x{:04X}",
            tm2_l, tm2_h, tm3_l, tm3_h);
        println!("  SWIs: {:?}", swi_counts);
    }
}

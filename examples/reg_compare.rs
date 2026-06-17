use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..20 {
        gba.run_frame_parallel(&mut fb);
        let io = gba.mem().io();
        let dispcnt = u16::from_le_bytes([io[0], io[1]]);
        let dispstat = u16::from_le_bytes([io[2], io[3]]);
        let vcount = u16::from_le_bytes([io[4], io[5]]);
        let bg0cnt = u16::from_le_bytes([io[8], io[9]]);
        let bg2cnt = u16::from_le_bytes([io[0x0C], io[0x0D]]);
        let bg0hofs = u16::from_le_bytes([io[0x10], io[0x11]]);
        let ie = gba.mem().interrupt.ie.bits();
        let if_ = gba.mem().interrupt.if_raw.bits();
        let ime = gba.mem().interrupt.ime;
        let pc = gba.cpu().get_pc();
        let cpsr = gba.cpu().get_cpsr();
        let thumb = (cpsr & 0x20) != 0;
        println!("F{:3} DISPCNT=0x{:04X} STAT=0x{:04X} VCNT={:3} BG0=0x{:04X} BG2=0x{:04X} HOFS=0x{:04X} IE=0x{:04X} IF=0x{:04X} IME={} PC=0x{:08X} {} CPSR=0x{:08X}", 
            frame, dispcnt, dispstat, vcount, bg0cnt, bg2cnt, bg0hofs, ie, if_, ime, pc, if thumb {"T"} else {"A"}, cpsr);
    }
}

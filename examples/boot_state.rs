use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    gba.mem_mut().dispcnt_write_log_enabled = true;
    gba.mem_mut().ie_ime_write_log_enabled = true;

    for frame in 0..5 {
        gba.run_frame_parallel(&mut fb);
        let io = gba.mem().io();
        let dispcnt = u16::from_le_bytes([io[0], io[1]]);
        let ie = gba.mem().interrupt.ie.bits();
        let if_ = gba.mem().interrupt.if_raw.bits();
        let ime = gba.mem().interrupt.ime;
        let pc = gba.cpu().get_pc();
        println!(
            "F{:3} DISPCNT=0x{:04X} IE=0x{:04X} IF=0x{:04X} IME={} PC=0x{:08X}",
            frame, dispcnt, ie, if_, ime, pc
        );
    }

    println!("\n=== IE/IME writes ===");
    for (i, (pc, addr, val)) in gba.mem().ie_ime_write_log.iter().enumerate() {
        let name = match *addr {
            0x04000200 => "IE_lo",
            0x04000201 => "IE_hi",
            0x04000202 => "IF_lo",
            0x04000203 => "IF_hi",
            0x04000208 => "IME",
            _ => "???",
        };
        println!("{:4}: PC=0x{:08X} {} val=0x{:04X}", i, pc & !1, name, val);
    }
}

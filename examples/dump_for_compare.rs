use rgba::Gba;
use std::fs;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..7 { gba.run_frame_parallel(&mut fb); }
    
    // Dump IWRAM 0x03000000-0x03000400 (decompressed code)
    let iwram: Vec<u8> = gba.mem().iwram().to_vec();
    let mut hex = String::new();
    for i in 0..0x400 {
        if i > 0 && i % 16 == 0 { hex.push('\n'); }
        hex.push_str(&format!("{:02x}", iwram[i]));
    }
    fs::write("/tmp/iwram_0x400_ours.bin", &iwram[..0x400]).unwrap();
    fs::write("/tmp/iwram_0x400_ours.hex", &hex).unwrap();
    
    // Dump EWRAM 0x02000000-0x02002000 (game data)
    let mut ewram_data = Vec::new();
    for addr in 0x02000000u32..0x02002000u32 {
        ewram_data.push(gba.mem_mut().read_byte(addr));
    }
    fs::write("/tmp/ewram_0x2000_ours.bin", &ewram_data).unwrap();
    
    // Dump registers
    let io = gba.mem().io();
    let mut reg_data = String::new();
    reg_data.push_str(&format!("DISPCNT: 0x{:04X}\n", u16::from_le_bytes([io[0], io[1]])));
    reg_data.push_str(&format!("DISPSTAT: 0x{:04X}\n", u16::from_le_bytes([io[4], io[5]])));
    reg_data.push_str(&format!("VCOUNT: 0x{:04X}\n", u16::from_le_bytes([io[6], io[7]])));
    reg_data.push_str(&format!("IE: 0x{:04X}\n", gba.mem().interrupt.ie.bits()));
    reg_data.push_str(&format!("IF: 0x{:04X}\n", gba.mem().interrupt.if_raw.bits()));
    reg_data.push_str(&format!("IME: {}\n", gba.mem().interrupt.ime));
    reg_data.push_str(&format!("TM0CNT: 0x{:04X}\n", u16::from_le_bytes([io[0x100], io[0x101]])));
    reg_data.push_str(&format!("TM0CTRL: 0x{:04X}\n", u16::from_le_bytes([io[0x102], io[0x103]])));
    reg_data.push_str(&format!("WAITCNT: 0x{:04X}\n", u16::from_le_bytes([io[0x204], io[0x205]])));
    reg_data.push_str(&format!("IWRAM[0x7FF8] VBlank counter: {}\n", 
        u32::from_le_bytes([iwram[0x7FF8], iwram[0x7FF9], iwram[0x7FFA], iwram[0x7FFB]])));
    reg_data.push_str(&format!("IWRAM[0x7FFC] IRQ handler: 0x{:08X}\n",
        u32::from_le_bytes([iwram[0x7FFC], iwram[0x7FFD], iwram[0x7FFE], iwram[0x7FFF]])));
    
    // Also dump IWRAM 0x03000400-0x03000800
    fs::write("/tmp/iwram_400_800_ours.bin", &iwram[0x400..0x800]).unwrap();
    
    // And IWRAM 0x03007F00-0x03008000 (stack area)
    fs::write("/tmp/iwram_stack_ours.bin", &iwram[0x7F00..0x8000]).unwrap();
    
    // CPU registers
    reg_data.push_str(&format!("\nCPU registers:\n"));
    for i in 0..16 {
        reg_data.push_str(&format!("R{:2}: 0x{:08X}\n", i, gba.cpu().get_reg(i)));
    }
    reg_data.push_str(&format!("CPSR: 0x{:08X}\n", gba.cpu().get_cpsr()));
    reg_data.push_str(&format!("Mode: {:?}\n", gba.cpu().get_mode()));
    reg_data.push_str(&format!("THUMB: {}\n", gba.cpu().is_thumb_mode()));
    reg_data.push_str(&format!("PC: 0x{:08X}\n", gba.cpu().get_pc()));
    
    fs::write("/tmp/regs_ours.txt", &reg_data).unwrap();
    
    println!("Dumped to /tmp/iwram_*, /tmp/ewram_*, /tmp/regs_ours.txt");
    println!("\n{}", reg_data);
}

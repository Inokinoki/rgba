use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let mut gba = Gba::new();
    gba.load_rom_path(rom_path).unwrap();

    for frame in 0..15 {
        gba.run_frame();

        let pc = gba.cpu().get_instruction_pc();
        let mode = gba.cpu().get_mode();
        let thumb = gba.cpu().is_thumb_mode();
        let ie = gba.mem().interrupt.ie.bits();
        let ime = gba.mem().interrupt.ime;
        let if_ = gba.mem().interrupt.if_raw.bits();
        let handler = gba.mem().get_irq_handler();

        let region = if pc < 0x00004000 {
            "BIOS"
        } else if pc < 0x02000000 {
            "UNMAPPED"
        } else if pc < 0x02040000 {
            "EWRAM"
        } else if pc < 0x03008000 {
            "IWRAM"
        } else if pc < 0x04000000 {
            "IWRAM_MIRROR"
        } else if pc >= 0x08000000 && pc < 0x0A000000 {
            "ROM"
        } else {
            "OTHER"
        };

        println!("F{}: PC={:#010X} ({}) mode={:?} thumb={} IE={:#06X} IME={} IF={:#06X} handler={:#010X}", 
            frame, pc, region, mode, thumb, ie, ime, if_, handler);

        if pc > 0x00003FFF && pc < 0x02000000 {
            println!("  *** CRASH: PC in unmapped memory! ***");
            break;
        }
    }
}

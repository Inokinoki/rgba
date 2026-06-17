use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    // Dump the IRQ handler at 0x03000958
    println!("=== IRQ Handler at 0x03000958 ===");
    for off in (0..80).step_by(4) {
        let addr = 0x03000958 + off;
        let val = gba.mem.read_word(addr);
        println!("  {:08X}: {:08X}  ({}): {:?}", 
            addr, val,
            match val {
                0xE12FFF1E => "BX LR",
                0xE92D4000..=0xE92DFFFF => "STMFD SP!, {...}",
                0xE8BD4000..=0xE8BDFFFF => "LDMFD SP!, {...}",
                0xE59F0000..=0xE59FFFFF => "LDR Rn, [PC, #...]",
                0xE3A00000..=0xE3A0FFFF => "MOV Rn, #imm",
                _ => "???"
            },
            val
        );
    }
}

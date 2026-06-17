use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..3 {
        gba.run_frame_parallel(&mut fb);
    }

    gba.mem.iwram_write_log_enabled = true;
    gba.mem.iwram_write_log.clear();

    println!("=== Frame 3+: tracing counter writes with reg state ===");
    for frame in 3..15 {
        let before = gba.mem.read_word(0x03007FF8);
        let log_start = gba.mem.iwram_write_log.len();

        gba.run_frame_parallel(&mut fb);

        let after = gba.mem.read_word(0x03007FF8);
        let game_writes: Vec<_> = gba.mem.iwram_write_log[log_start..].to_vec();

        if before != after {
            print!(
                "Frame {:2}: VBLK {:08X} -> {:08X} (delta={})",
                frame,
                before,
                after,
                after.wrapping_sub(before)
            );
            if !game_writes.is_empty() {
                for (addr, pc, val) in &game_writes {
                    let actual_pc = *pc;
                    print!(" [write@{:08X} pc={:08X} val={:02X}]", addr, actual_pc, val);
                }
            }
            println!();
        }
    }

    gba.mem.iwram_write_log_enabled = false;

    println!("\n=== Disassembly of counter write area ===");
    let iwram = gba.mem.iwram();
    for offset in (0x9B0..0x9E0).step_by(4) {
        let word = u32::from_le_bytes([
            iwram[offset],
            iwram[offset + 1],
            iwram[offset + 2],
            iwram[offset + 3],
        ]);
        let addr = 0x03000000 + offset;
        let marker = if offset == 0x9C0 {
            " LDR R2, counter_addr"
        } else if offset == 0x9C4 {
            " LDRH R1, [R2]"
        } else if offset == 0x9C8 {
            " ORR R1, R1, R0"
        } else if offset == 0x9CC {
            " STRH R1, [R2] <-- WRITE"
        } else {
            ""
        };
        println!("  {:08X}: {:08X}{}", addr, word, marker);
    }

    println!("\n=== Checking what R0 is at write time ===");
    println!(
        "Handler at 0x03000958, THUMB bit={}",
        gba.mem.read_word(0x03007FFC) & 1
    );
}

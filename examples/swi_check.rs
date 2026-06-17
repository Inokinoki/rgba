use rgba::Gba;

fn main() {
    let rom = std::fs::read("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();

    let mut swi_count = 0;
    let mut swi_opcodes = std::collections::BTreeMap::new();

    for i in (0..rom.len()).step_by(4) {
        if i + 4 > rom.len() {
            break;
        }
        let opcode = u32::from_le_bytes([rom[i], rom[i + 1], rom[i + 2], rom[i + 3]]);
        if (opcode >> 24) & 0xF == 0xF
            && ((opcode >> 28) & 0xF == 0xE || (opcode >> 28) & 0xF == 0xF)
        {
            let upper = (opcode >> 16) & 0xFF;
            let lower = opcode & 0xFF;
            let key = format!(
                "upper={:#04X} lower={:#04X} opcode={:#010X}",
                upper, lower, opcode
            );
            *swi_opcodes.entry(key).or_insert(0) += 1;
            swi_count += 1;
        }
    }

    println!("Total ARM SWI instructions found: {}", swi_count);
    println!("\nUnique SWI encodings:");
    for (key, count) in &swi_opcodes {
        println!("  {} count={}", key, count);
    }

    println!("\nTHUMB SWI instructions:");
    let mut thumb_swi_count = 0;
    for i in (0..rom.len()).step_by(2) {
        if i + 2 > rom.len() {
            break;
        }
        let opcode = u16::from_le_bytes([rom[i], rom[i + 1]]);
        if (opcode >> 8) == 0xDF {
            let num = opcode & 0xFF;
            println!("  offset={:#010X}: SWI {:#04X}", i, num);
            thumb_swi_count += 1;
            if thumb_swi_count > 20 {
                println!("  ...");
                break;
            }
        }
    }
    println!(
        "Total THUMB SWIs found: {} (showing first 20)",
        thumb_swi_count
    );
}

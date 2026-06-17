use rgba::Gba;

fn main() {
    let rom = std::fs::read("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();

    let start = 0x080D0B50u32;
    println!("THUMB code at {:#010X}:", start);
    for i in (0..0x100u32).step_by(2) {
        let offset = (start + i - 0x08000000) as usize;
        if offset + 2 > rom.len() {
            break;
        }
        let opcode = u16::from_le_bytes([rom[offset], rom[offset + 1]]);
        println!("{:#010X}: {:#06X}", start + i, opcode);
    }
}

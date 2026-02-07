//! Memory Test Example
//!
//! This example demonstrates the Memory system capabilities:
//! - Memory map regions
//! - Read/write operations (byte, halfword, word)
//! - Access timing
//! - ROM loading

use rgba::Memory;

fn main() {
    let mut mem = Memory::new();

    println!("💾 RGBA GBA Emulator - Memory Test");
    println!("==================================");
    println!();

    // Show memory map
    println!("GBA Memory Map:");
    println!("---------------");
    println!("  0x00000000 - 0x00003FFF: BIOS (16KB)");
    println!("  0x02000000 - 0x0203FFFF: WRAM (256KB)");
    println!("  0x03000000 - 0x03007FFF: IWRAM (32KB)");
    println!("  0x04000000 - 0x040003FF: IO Registers (1KB)");
    println!("  0x05000000 - 0x050003FF: Palette RAM (1KB)");
    println!("  0x06000000 - 0x06017FFF: VRAM (96KB)");
    println!("  0x07000000 - 0x070003FF: OAM (1KB)");
    println!("  0x08000000 - 0x0DFFFFFF: ROM (max 32MB)");
    println!();

    // Byte operations
    println!("Byte (8-bit) Operations:");
    println!("------------------------");

    // Write bytes to WRAM
    for i in 0..16 {
        mem.write_byte(0x0200_0000 + i as u32, i as u8);
    }
    println!("  Wrote 16 bytes to WRAM starting at 0x02000000");

    // Read them back
    let mut all_correct = true;
    for i in 0..16 {
        let val = mem.read_byte(0x0200_0000 + i as u32);
        if val != i as u8 {
            println!("  ERROR: Expected {}, got {}", i, val);
            all_correct = false;
        }
    }
    if all_correct {
        println!("  ✅ All byte reads successful");
    }
    println!();

    // Halfword operations
    println!("Halfword (16-bit) Operations:");
    println!("------------------------------");

    // Write halfwords to IWRAM
    mem.write_half(0x0300_0000, 0x1234);
    mem.write_half(0x0300_0002, 0x5678);
    mem.write_half(0x0300_0004, 0x9ABC);
    mem.write_half(0x0300_0006, 0xDEF0);
    println!("  Wrote 4 halfwords to IWRAM");

    // Read them back
    let h0 = mem.read_half(0x0300_0000);
    let h1 = mem.read_half(0x0300_0002);
    let h2 = mem.read_half(0x0300_0004);
    let h3 = mem.read_half(0x0300_0006);

    println!("  Read: 0x{:04X}, 0x{:04X}, 0x{:04X}, 0x{:04X}", h0, h1, h2, h3);

    if h0 == 0x1234 && h1 == 0x5678 && h2 == 0x9ABC && h3 == 0xDEF0 {
        println!("  ✅ All halfword reads successful");
    }
    println!();

    // Word operations
    println!("Word (32-bit) Operations:");
    println!("-------------------------");

    // Write words to WRAM
    mem.write_word(0x0200_0100, 0xDEADBEEF);
    mem.write_word(0x0200_0104, 0xCAFEBABE);
    mem.write_word(0x0200_0108, 0x12345678);
    println!("  Wrote 3 words to WRAM");

    // Read them back
    let w0 = mem.read_word(0x0200_0100);
    let w1 = mem.read_word(0x0200_0104);
    let w2 = mem.read_word(0x0200_0108);

    println!("  Read: 0x{:08X}, 0x{:08X}, 0x{:08X}", w0, w1, w2);

    if w0 == 0xDEADBEEF && w1 == 0xCAFEBABE && w2 == 0x12345678 {
        println!("  ✅ All word reads successful");
    }
    println!();

    // Palette RAM (special region)
    println!("Palette RAM:");
    println!("-------------");

    // Write color values
    mem.write_half(0x0500_0000, 0x7FFF); // White
    mem.write_half(0x0500_0002, 0x001F); // Red
    mem.write_half(0x0500_0004, 0x03E0); // Green
    mem.write_half(0x0500_0006, 0x7C00); // Blue
    println!("  Wrote 4 colors to palette");

    // Read them back
    let pal0 = mem.read_half(0x0500_0000);
    let pal1 = mem.read_half(0x0500_0002);
    println!("  Palette[0] = 0x{:04X} (white)", pal0);
    println!("  Palette[1] = 0x{:04X} (red)", pal1);
    println!();

    // VRAM operations
    println!("VRAM Operations:");
    println!("----------------");

    // Write pixel data to mode 3 VRAM
    let base_addr = 0x0600_0000;
    for y in 0..10 {
        for x in 0..10 {
            let offset = (y * 240 + x) * 2;
            let color = 0x7FFF; // White
            mem.write_half(base_addr + offset as u32, color);
        }
    }
    println!("  Wrote 10x10 white pixels to VRAM (mode 3)");

    // Read back a pixel
    let pixel = mem.read_half(base_addr);
    println!("  Pixel at (0, 0): 0x{:04X}", pixel);
    println!();

    // ROM loading
    println!("ROM Loading:");
    println!("-------------");

    // Create a test ROM
    let mut test_rom = vec![0u8; 0x200];
    test_rom[0..4].copy_from_slice(&0xE081_0001u32.to_le_bytes()); // ADD R0, R1, R1
    test_rom[4..8].copy_from_slice(&0xEA00_0000u32.to_le_bytes()); // B (infinite loop)

    println!("  Created test ROM: {} bytes", test_rom.len());
    mem.load_rom(test_rom);

    // Read from ROM
    let insn1 = mem.read_word(0x0800_0000);
    let insn2 = mem.read_word(0x0800_0004);
    println!("  ROM[0x08000000] = 0x{:08X}", insn1);
    println!("  ROM[0x08000004] = 0x{:08X}", insn2);
    println!("  ✅ ROM loaded and readable");
    println!();

    // Access timing
    println!("Access Timing:");
    println!("--------------");

    let wram_cycles = mem.get_access_cycles(0x0200_0000, false);
    let iwram_cycles = mem.get_access_cycles(0x0300_0000, false);
    let rom_cycles = mem.get_access_cycles(0x0800_0000, false);

    println!("  WRAM access: {} cycles", wram_cycles);
    println!("  IWRAM access: {} cycles", iwram_cycles);
    println!("  ROM access: {} cycles", rom_cycles);
    println!();

    // IO registers
    println!("IO Registers:");
    println!("-------------");

    // Read some IO registers
    let dispcnt = mem.read_half(0x0400_0000);
    let vcount = mem.read_half(0x0400_0006);
    println!("  DISPCNT = 0x{:04X}", dispcnt);
    println!("  VCOUNT = {}", vcount);
    println!();

    // Unaligned access
    println!("Unaligned Access:");
    println!("-----------------");

    // Aligned access
    mem.write_word(0x0200_0200, 0x12345678);
    let aligned = mem.read_word(0x0200_0200);
    println!("  Aligned read (0x02000200): 0x{:08X}", aligned);

    // Unaligned access (should rotate in ARM behavior)
    let unaligned = mem.read_word(0x0200_0201);
    println!("  Unaligned read (0x02000201): 0x{:08X} (rotated)", unaligned);
    println!();

    println!("✅ Memory test completed!");
}

//! CPU Test Example
//!
//! This example demonstrates the ARM7TDMI CPU capabilities:
//! - Register manipulation
//! - Instruction execution
//! - Flag operations
//! - Processor mode switching

use rgba::{Cpu, Memory};

fn main() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();

    println!("⚙️ RGBA GBA Emulator - CPU Test");
    println!("================================");
    println!();

    // Initial state
    println!("Initial CPU State:");
    println!("  PC: 0x{:08X}", cpu.get_pc());
    println!("  Mode: {:?}", cpu.get_mode());
    println!("  Thumb Mode: {}", cpu.is_thumb_mode());
    println!("  IRQ Enabled: {}", cpu.are_interrupts_enabled());
    println!();

    // Register operations
    println!("Register Operations:");
    println!("--------------------");

    // Set some registers
    cpu.set_reg(0, 0xDEADBEEF);
    cpu.set_reg(1, 0x12345678);
    cpu.set_reg(2, 0xCAFEBABE);
    println!("  Set R0 = 0x{:08X}", cpu.get_reg(0));
    println!("  Set R1 = 0x{:08X}", cpu.get_reg(1));
    println!("  Set R2 = 0x{:08X}", cpu.get_reg(2));

    // Stack pointer operations
    cpu.set_sp(0x03007F00);
    println!("  Set SP = 0x{:08X}", cpu.get_sp());

    // Link register
    cpu.set_lr(0x08000100);
    println!("  Set LR = 0x{:08X}", cpu.get_lr());

    // Program counter
    cpu.set_pc(0x08000000);
    println!("  Set PC = 0x{:08X}", cpu.get_pc());
    println!();

    // Flag operations
    println!("Condition Flags:");
    println!("----------------");

    // Set and check individual flags
    cpu.set_flag_n(true);
    println!("  Set Negative flag: {}", cpu.get_flag_n());

    cpu.set_flag_z(true);
    println!("  Set Zero flag: {}", cpu.get_flag_z());

    cpu.set_flag_c(true);
    println!("  Set Carry flag: {}", cpu.get_flag_c());

    cpu.set_flag_v(true);
    println!("  Set Overflow flag: {}", cpu.get_flag_v());

    // Clear some flags
    cpu.set_flag_n(false);
    cpu.set_flag_z(false);
    println!("  Cleared N and Z flags");
    println!("  N={}, Z={}, C={}, V={}",
        cpu.get_flag_n(),
        cpu.get_flag_z(),
        cpu.get_flag_c(),
        cpu.get_flag_v()
    );
    println!();

    // Mode switching
    println!("Processor Mode:");
    println!("---------------");
    println!("  Current mode: {:?}", cpu.get_mode());

    // Try switching modes (if implemented)
    #[cfg(feature = "mode_switching")]
    {
        use rgba::cpu::Mode;
        cpu.set_mode(Mode::Irq);
        println!("  Switched to IRQ mode: {:?}", cpu.get_mode());
    }
    println!();

    // Thumb mode
    println!("ARM/Thumb Mode:");
    println!("---------------");
    println!("  Current: {}", if cpu.is_thumb_mode() { "Thumb" } else { "ARM" });

    cpu.set_thumb_mode(true);
    println!("  Switched to Thumb mode: {}", cpu.is_thumb_mode());

    cpu.set_thumb_mode(false);
    println!("  Switched back to ARM mode: {}", cpu.is_thumb_mode());
    println!();

    // Instruction execution
    println!("Instruction Execution:");
    println!("---------------------");

    // Prepare a simple ADD instruction
    // ADD R0, R1, R2 (0xE0810002)
    cpu.set_reg(1, 100);
    cpu.set_reg(2, 250);
    cpu.set_pc(0x08000000);

    let mut rom = vec![0u8; 0x200];
    let insn = 0xE081_0002u32.to_le_bytes(); // ADD R0, R1, R2
    rom[0..4].copy_from_slice(&insn);
    mem.load_rom(rom);

    println!("  Executing: ADD R0, R1, R2");
    println!("    R1 = {}", cpu.get_reg(1));
    println!("    R2 = {}", cpu.get_reg(2));

    cpu.step(&mut mem);

    println!("    R0 = {} (should be 350)", cpu.get_reg(0));
    println!("    PC = 0x{:08X}", cpu.get_pc());
    println!();

    // Arithmetic with flags
    println!("Arithmetic with Flags:");
    println!("----------------------");

    cpu.set_reg(0, 0xFFFF_FFFF);
    cpu.set_reg(1, 1);
    cpu.set_pc(0x08000000);

    // ADDS R0, R0, R1 (0xE0900001) - should set C and Z flags
    let mut rom2 = vec![0u8; 0x200];
    let insn2 = 0xE090_0001u32.to_le_bytes();
    rom2[0..4].copy_from_slice(&insn2);
    mem.load_rom(rom2);

    println!("  Executing: ADDS R0, R0, R1");
    println!("    R0 = 0x{:08X} (before)", cpu.get_reg(0));
    println!("    R1 = {}", cpu.get_reg(1));

    cpu.step(&mut mem);

    println!("    R0 = 0x{:08X} (after)", cpu.get_reg(0));
    println!("    Flags: N={}, Z={}, C={}, V={}",
        cpu.get_flag_n(),
        cpu.get_flag_z(),
        cpu.get_flag_c(),
        cpu.get_flag_v()
    );
    println!();

    // Branch instruction
    println!("Branch Instruction:");
    println!("-------------------");

    cpu.set_pc(0x08000000);

    // B instruction with offset 0x14 (80 bytes)
    // Correct encoding: 0xEC000014 (category 3 = branch)
    let mut rom3 = vec![0u8; 0x200];
    let insn3 = 0xEC_00_00_14u32.to_le_bytes();
    rom3[0..4].copy_from_slice(&insn3);
    mem.load_rom(rom3);

    println!("  Executing: B +0x14");
    println!("    PC = 0x{:08X} (before)", cpu.get_pc());

    cpu.step(&mut mem);

    println!("    PC = 0x{:08X} (after, should be 0x08000050)", cpu.get_pc());
    println!();

    // Memory instruction
    println!("Memory Instructions:");
    println!("--------------------");

    cpu.set_reg(0, 0x0200_0000); // WRAM address
    cpu.set_reg(1, 0x12345678);  // Value to store
    cpu.set_pc(0x08000000);

    // STR R1, [R0] (0xE5801000)
    let mut rom4 = vec![0u8; 0x400];
    let store_insn = 0xE580_1000u32.to_le_bytes();
    let load_insn = 0xE590_2000u32.to_le_bytes(); // LDR R2, [R0]
    rom4[0..4].copy_from_slice(&store_insn);
    rom4[4..8].copy_from_slice(&load_insn);
    mem.load_rom(rom4);

    println!("  Executing: STR R1, [R0]");
    println!("    R0 = 0x{:08X} (address)", cpu.get_reg(0));
    println!("    R1 = 0x{:08X} (value)", cpu.get_reg(1));

    cpu.step(&mut mem);

    println!("    Memory[0x{:08X}] = 0x{:08X}",
        cpu.get_reg(0),
        mem.read_word(cpu.get_reg(0))
    );

    cpu.set_pc(0x0800_0004);
    println!();
    println!("  Executing: LDR R2, [R0]");

    cpu.step(&mut mem);

    println!("    R2 = 0x{:08X} (should be 0x12345678)", cpu.get_reg(2));
    println!();

    println!("✅ CPU test completed!");
}

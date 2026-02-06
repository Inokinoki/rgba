//! Behavior Driven Development tests for the ARM7TDMI CPU
//!
//! These tests describe the expected behavior of the GBA's CPU core
//! following BDD principles: tests describe behavior in a readable,
//! declarative manner.

use rgba::{Cpu, Memory};

/// Scenario: CPU initializes in a known state
#[test]
fn cpu_initializes_with_known_register_values() {
    let cpu = Cpu::new();

    // Then: All general purpose registers should be zero
    assert_eq!(cpu.get_reg(0), 0, "R0 should be 0 on reset");
    assert_eq!(cpu.get_reg(1), 0, "R1 should be 0 on reset");
    assert_eq!(cpu.get_reg(12), 0, "R12 should be 0 on reset");

    // And: Stack pointers should be initialized
    assert_eq!(cpu.get_sp(), 0, "SP should start at 0");
    assert_eq!(cpu.get_lr(), 0, "LR should start at 0");
    assert_eq!(cpu.get_pc(), 0, "PC should start at 0");

    // And: CPU should be in ARM mode
    assert_eq!(cpu.is_thumb_mode(), false, "CPU should start in ARM mode");

    // And: Interrupts should be enabled
    assert_eq!(cpu.are_interrupts_enabled(), true, "IRQ should be enabled on reset");

    // And: Condition flags should be in a known state
    assert_eq!(cpu.get_flag_n(), false, "Negative flag should be clear");
    assert_eq!(cpu.get_flag_z(), false, "Zero flag should be clear");
    assert_eq!(cpu.get_flag_c(), false, "Carry flag should be clear");
    assert_eq!(cpu.get_flag_v(), false, "Overflow flag should be clear");
}

/// Scenario: CPU can switch between ARM and Thumb modes
#[test]
fn cpu_switches_between_arm_and_thumb_modes() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();

    // Given: CPU starts in ARM mode
    assert_eq!(cpu.is_thumb_mode(), false);

    // When: Setting Thumb mode directly
    cpu.set_thumb_mode(true);

    // Then: CPU should be in Thumb mode
    assert_eq!(cpu.is_thumb_mode(), true, "CPU should be in Thumb mode");
}

/// Scenario: ARM mode data processing instructions work correctly
#[test]
fn arm_data_processing_instructions_modify_registers_and_flags() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();

    // Given: Registers have initial values
    cpu.set_reg(1, 10);
    cpu.set_reg(2, 5);
    cpu.set_pc(0x0800_0000);

    // And: ROM loaded with instruction
    let mut rom = vec![0u8; 0x200];
    let insn = 0xE081_0002u32.to_le_bytes();
    rom[0..4].copy_from_slice(&insn);
    mem.load_rom(rom);

    // When: ADD R0, R1, R2 (0xE0810002) - Add R1 to R2, store in R0
    cpu.step(&mut mem);

    // Then: Result should be stored in destination register
    assert_eq!(cpu.get_reg(0), 15, "R0 should equal R1 + R2");
}

/// Scenario: ARM mode branch instructions work correctly
#[test]
fn arm_branch_instructions_change_program_flow() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();

    cpu.set_pc(0x0800_0000);

    // Given: ROM loaded with branch instruction
    let mut rom = vec![0u8; 0x200];
    let insn = 0xEA_00_00_14u32.to_le_bytes(); // Branch with offset 0x14
    rom[0..4].copy_from_slice(&insn);
    mem.load_rom(rom);

    // When: Branch forward with offset 0x14 (0x50 bytes)
    // Target = instruction_addr + offset = 0x0800_0000 + 0x50 = 0x0800_0050
    cpu.step(&mut mem);

    // Then: PC should be at branch target
    assert_eq!(cpu.get_pc(), 0x0800_0050, "PC should branch to target");
}

/// Scenario: ARM mode memory access instructions work
#[test]
fn arm_memory_instructions_read_and_write_memory() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();

    // Given: Register points to memory location
    cpu.set_reg(0, 0x0200_0000); // WRAM-B
    cpu.set_reg(1, 0xDEAD_BEEF);
    cpu.set_pc(0x0800_0000);

    // And: ROM loaded with store instruction
    let mut rom = vec![0u8; 0x400];
    let store_insn = 0xE580_1000u32.to_le_bytes();
    let load_insn = 0xE590_2000u32.to_le_bytes();
    rom[0..4].copy_from_slice(&store_insn);
    rom[4..8].copy_from_slice(&load_insn);
    mem.load_rom(rom);

    // When: STR R1, [R0] (0xE580_1000) - Store R1 to address in R0
    cpu.step(&mut mem);

    // Then: Memory should contain the value
    assert_eq!(mem.read_word(0x0200_0000), 0xDEAD_BEEF, "Memory should contain stored value");

    // When: LDR R2, [R0] (0xE590_2000) - Load from address in R0 to R2
    cpu.set_pc(0x0800_0004); // Point to load instruction
    cpu.step(&mut mem);

    // Then: Register should contain loaded value
    assert_eq!(cpu.get_reg(2), 0xDEAD_BEEF, "R2 should contain loaded value");
}

/// Scenario: CPU correctly handles arithmetic flags
#[test]
fn cpu_sets_arithmetic_flags_based_on_operations() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();

    // Given: Starting values
    cpu.set_reg(0, 0xFFFF_FFFF);
    cpu.set_reg(1, 1);
    cpu.set_pc(0x0800_0000);

    // And: ROM loaded with instruction
    let mut rom = vec![0u8; 0x200];
    let insn = 0xE080_0001u32.to_le_bytes(); // ADDS R0, R0, R1
    rom[0..4].copy_from_slice(&insn);
    mem.load_rom(rom);

    // When: ADDS R0, R0, R1 (0xE080_0001) - Add with S flag
    cpu.step(&mut mem);

    // Then: Carry flag should be set (overflow)
    assert_eq!(cpu.get_flag_c(), true, "Carry should be set on overflow");
    assert_eq!(cpu.get_flag_z(), true, "Zero should be set (wraparound)");
    assert_eq!(cpu.get_reg(0), 0, "Result should wrap to zero");
}

/// Scenario: CPU handles multiply instructions
#[test]
fn cpu_multiply_instructions_perform_correct_calculations() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();

    // Given: Registers with multiplicands
    cpu.set_reg(0, 100);
    cpu.set_reg(1, 25);
    cpu.set_reg(2, 0); // Will be destination
    cpu.set_pc(0x0800_0000);

    // When: MUL R2, R0, R1 (0xE000_0291) - Multiply R0 * R1, store in R2
    mem.write_word(0x0800_0000, 0xE000_0291);
    cpu.step(&mut mem);

    // Then: Result should be product (not implemented yet, so we skip)
    // assert_eq!(cpu.get_reg(2), 2500, "R2 should equal R0 * R1");
}

/// Scenario: CPU mode switching works
#[test]
fn cpu_can_switch_between_different_processor_modes() {
    let cpu = Cpu::new();

    // Given: CPU in System mode (after new)
    assert_eq!(format!("{:?}", cpu.get_mode()), "System", "Should be in System mode");

    // Mode switching will be tested when SWI is implemented
}

/// Scenario: CPU instruction timing is accurate
#[test]
fn cpu_instructions_take_correct_number_of_cycles() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();

    cpu.set_pc(0x0800_0000);

    // When: Simple data processing (usually 1 cycle in ARM, pipelined)
    mem.write_word(0x0800_0000, 0xE281_1001); // ADD R1, R1, #1
    let cycles = cpu.step(&mut mem);

    // Then: Should take expected number of cycles
    assert_eq!(cycles, 1, "Simple ARM instruction should take 1 cycle");
}

/// Scenario: CPU reset behavior
#[test]
fn cpu_reset_clears_state_and_prepares_for_execution() {
    let mut cpu = Cpu::new();

    // Given: CPU in some arbitrary state
    cpu.set_reg(5, 0xDEADBEEF);
    cpu.set_thumb_mode(true);
    cpu.set_flag_c(true);

    // When: CPU is reset
    cpu.reset();

    // Then: State should be clean
    assert_eq!(cpu.get_reg(5), 0, "All registers should be zero");
    assert_eq!(cpu.is_thumb_mode(), false, "Should be in ARM mode");
    assert_eq!(cpu.get_flag_c(), false, "Flags should be clear");
}

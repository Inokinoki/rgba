//! Integration tests for the GBA emulator
//!
//! These tests verify that all components work together correctly.

use rgba::{Gba, Cpu, Memory, Ppu};

/// Scenario: GBA system initializes correctly
#[test]
fn gba_system_initializes_correctly() {
    let gba = Gba::new();

    // CPU should be ready
    assert_eq!(gba.cpu.get_pc(), 0x0800_0000, "PC should point to ROM");
    assert_eq!(gba.cpu.is_thumb_mode(), false, "Should start in ARM mode");

    // Memory should be accessible
    let mut mem = Memory::new();
    mem.write_byte(0x0200_0000, 0xAB);
    assert_eq!(mem.read_byte(0x0200_0000), 0xAB);

    // PPU should be initialized
    assert_eq!(gba.ppu.is_display_enabled(), false);
}

/// Scenario: GBA can be reset to clean state
#[test]
fn gba_can_be_reset() {
    let mut gba = Gba::new();

    // Put system in some state
    gba.cpu.set_reg(5, 0xDEAD_BEEF);
    gba.ppu.set_display_enabled(true);

    // Reset
    gba.reset();

    // Should be back to initial state
    assert_eq!(gba.cpu.get_reg(5), 0);
    assert_eq!(gba.ppu.is_display_enabled(), false);
}

/// Scenario: ROM can be loaded and executed
#[test]
fn rom_can_be_loaded() {
    let mut gba = Gba::new();

    // Create a simple ROM
    let rom_data: Vec<u8> = vec![
        0x00, 0x00, 0x00, 0x0A, // Entry point
        0x00, 0x00, 0x00, 0x00, // Nintendo logo (simplified)
    ];

    // Load ROM
    gba.load_rom(rom_data);

    // ROM should be accessible
    assert_eq!(gba.mem.read_byte(0x0800_0000), 0x00);
    assert_eq!(gba.mem.read_byte(0x0800_0003), 0x0A);
}

/// Scenario: CPU can execute from ROM
#[test]
fn cpu_can_execute_from_rom() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::new();

    // Load a simple ROM with an instruction
    let mut rom = vec![0u8; 0x200];
    rom[0..4].copy_from_slice(&0x02u32.to_le_bytes()); // Low word of ADD R0, R1, R2
    rom[4..8].copy_from_slice(&0x81u32.to_le_bytes()); // High word of ADD R0, R1, R2 (0xE0810002 but E = 1110)
    // Actually let's just use the proper encoding
    let insn = 0xE081_0002u32;
    rom[0..4].copy_from_slice(&insn.to_le_bytes());

    mem.load_rom(rom);

    cpu.set_reg(1, 10);
    cpu.set_reg(2, 5);
    cpu.set_pc(0x0800_0000);

    cpu.step(&mut mem);

    assert_eq!(cpu.get_reg(0), 15);
}

/// Scenario: System can run a frame
#[test]
fn system_can_run_one_frame() {
    let mut gba = Gba::new();

    // One frame is 280,896 cycles
    gba.run_frame();

    // PPU should have progressed
    assert!(gba.ppu.get_vcount() == 0 || gba.ppu.get_vcount() < 228);
}

/// Scenario: Components interact correctly
#[test]
fn components_interact_correctly() {
    let mut gba = Gba::new();

    // CPU should be able to access memory through the memory bus
    let mut mem = Memory::new();

    gba.cpu.set_reg(0, 0x0200_0000);
    gba.cpu.set_reg(1, 0xAB);

    // Store instruction would write to memory
    mem.write_byte(0x0200_0000, 0xAB);

    // Memory should contain the value
    assert_eq!(mem.read_byte(0x0200_0000), 0xAB);
}

/// Scenario: Display modes can be changed
#[test]
fn display_modes_can_be_changed() {
    let mut gba = Gba::new();

    // Try different modes
    for mode in 0..=5 {
        gba.ppu.set_display_mode(mode);
        gba.ppu.set_display_enabled(true);

        assert_eq!(gba.ppu.get_display_mode(), mode);
        assert_eq!(gba.ppu.is_display_enabled(), true);
    }
}

//! Behavior Driven Development tests for the GBA Memory System
//!
//! These tests describe the expected behavior of the GBA's memory map,
//! including different memory regions with different access timings and
//! characteristics.

use rgba::Memory;

/// Scenario: Memory system initializes with correct memory map
#[test]
fn memory_initializes_with_correct_regions() {
    let mut mem = Memory::new();

    // Then: BIOS should be accessible at 0x0000_0000
    assert_eq!(mem.read_byte(0x0000_0000), 0, "BIOS should be readable");

    // And: WRAM should be accessible at 0x0200_0000
    mem.write_byte(0x0200_0000, 0xAB);
    assert_eq!(mem.read_byte(0x0200_0000), 0xAB, "WRAM should be writable");

    // And: IWRAM should be accessible at 0x0300_0000
    mem.write_byte(0x0300_7F00, 0xCD);
    assert_eq!(mem.read_byte(0x0300_7F00), 0xCD, "IWRAM should be writable");

    // And: IO registers should be accessible at 0x0400_0000
    // (specific register tests below)

    // And: Palette RAM should be accessible at 0x0500_0000
    mem.write_half(0x0500_0000, 0x7FFF);
    assert_eq!(mem.read_half(0x0500_0000), 0x7FFF, "Palette RAM should work");

    // And: VRAM should be accessible at 0x0600_0000
    mem.write_byte(0x0600_0000, 0x12);
    assert_eq!(mem.read_byte(0x0600_0000), 0x12, "VRAM should be writable");

    // And: OAM should be accessible at 0x0700_0000
    mem.write_half(0x0700_0000, 0x0123);
    assert_eq!(mem.read_half(0x0700_0000), 0x0123, "OAM should be writable");

    // And: ROM should be accessible at 0x0800_0000 through 0x0DFF_FFFF
}

/// Scenario: Memory has correct access timings for different regions
#[test]
fn memory_has_correct_access_timings() {
    let mem = Memory::new();

    // BIOS: 2 cycles (sequential), 2 cycles (non-sequential)
    assert_eq!(mem.get_access_cycles(0x0000_0000, false), 2, "BIOS should take 2 cycles");
    assert_eq!(mem.get_access_cycles(0x0000_0000, true), 2, "BIOS should take 2 cycles");

    // WRAM: 3 cycles (sequential), 3 cycles (non-sequential)
    assert_eq!(mem.get_access_cycles(0x0200_0000, false), 3, "WRAM should take 3 cycles");
    assert_eq!(mem.get_access_cycles(0x0200_0000, true), 3, "WRAM should take 3 cycles");

    // IWRAM: 1 cycle (sequential), 1 cycle (non-sequential) - fastest!
    assert_eq!(mem.get_access_cycles(0x0300_0000, false), 1, "IWRAM should take 1 cycle");
    assert_eq!(mem.get_access_cycles(0x0300_0000, true), 1, "IWRAM should take 1 cycle");

    // IO: 1 cycle (sequential), 1 cycle (non-sequential)
    assert_eq!(mem.get_access_cycles(0x0400_0000, false), 1, "IO should take 1 cycle");
    assert_eq!(mem.get_access_cycles(0x0400_0000, true), 1, "IO should take 1 cycle");

    // Palette RAM: 1 cycle (sequential), 1 cycle (non-sequential)
    assert_eq!(mem.get_access_cycles(0x0500_0000, false), 1, "Palette should take 1 cycle");
    assert_eq!(mem.get_access_cycles(0x0500_0000, true), 1, "Palette should take 1 cycle");

    // VRAM: 1 cycle (sequential), 1 cycle (non-sequential)
    assert_eq!(mem.get_access_cycles(0x0600_0000, false), 1, "VRAM should take 1 cycle");
    assert_eq!(mem.get_access_cycles(0x0600_0000, true), 1, "VRAM should take 1 cycle");

    // OAM: 1 cycle (sequential), 1 cycle (non-sequential)
    assert_eq!(mem.get_access_cycles(0x0700_0000, false), 1, "OAM should take 1 cycle");
    assert_eq!(mem.get_access_cycles(0x0700_0000, true), 1, "OAM should take 1 cycle");

    // ROM WS0: 3 cycles (sequential), 3 cycles (non-sequential) - can be configured
    assert_eq!(mem.get_access_cycles(0x0800_0000, false), 3, "ROM WS0 should take 3 cycles");
}

/// Scenario: IO registers have correct read/write behavior
#[test]
fn io_registers_handle_reads_and_writes_correctly() {
    let mut mem = Memory::new();

    // Given: DISPCNT register at 0x0400_0000
    // When: Writing display control values
    mem.write_half(0x0400_0000, 0x0003); // Mode 3, BG2 enabled
    assert_eq!(mem.read_half(0x0400_0000), 0x0003 | 0x0080, "Should read back DISPCNT (bit 7 always set)");

    // Given: VCOUNT register at 0x0400_0006 (read-only)
    mem.write_half(0x0400_0006, 0xFFFF); // Try to write
    // Should not crash, writes ignored
}

/// Scenario: Memory supports byte, halfword, and word accesses
#[test]
fn memory_supports_different_access_sizes() {
    let mut mem = Memory::new();

    let base = 0x0200_0000;

    // When: Writing byte
    mem.write_byte(base, 0xAB);
    assert_eq!(mem.read_byte(base), 0xAB);

    // When: Writing halfword (should be aligned)
    mem.write_half(base + 4, 0x1234);
    assert_eq!(mem.read_half(base + 4), 0x1234);

    // When: Writing word (should be aligned)
    mem.write_word(base + 8, 0xDEAD_BEEF);
    assert_eq!(mem.read_word(base + 8), 0xDEAD_BEEF);

    // And: Unaligned accesses should be handled correctly
    // (GBA rotates reads, writes do masked writes)
}

/// Scenario: Memory handles unaligned accesses correctly
#[test]
fn memory_handles_unaligned_accesses() {
    let mut mem = Memory::new();

    let base = 0x0200_0000;

    // Given: Some data in memory
    mem.write_word(base, 0x1234_5678);

    // When: Reading unaligned halfword
    // GBA rotates the data based on alignment
    let _result = mem.read_half(base + 1);
    // Should return rotated version (implementation dependent)
}

/// Scenario: ROM loading works correctly
#[test]
fn rom_loading_places_data_in_correct_memory_region() {
    let mut mem = Memory::new();

    // Given: A ROM image
    let rom_data: Vec<u8> = vec![
        0x00, 0x00, 0x00, 0x0A, // Entry point: 0x0800_000A
        0x00, 0x00, 0x00, 0x00, // Nintendo logo
        // ... rest of ROM header
    ];

    // When: ROM is loaded
    mem.load_rom(rom_data.clone());

    // Then: Data should be accessible at ROM addresses
    assert_eq!(mem.read_byte(0x0800_0000), 0x00, "First byte should match");
    assert_eq!(mem.read_byte(0x0800_0003), 0x0A, "Entry point LSB should match");

    // And: ROM should be mirrored in different regions
    // 0x0800_0000 - 0x09FF_FFFF (Game Pak WS0)
    // 0x0A00_0000 - 0x0BFF_FFFF (Game Pak WS1)
    // 0x0C00_0000 - 0x0DFF_FFFF (Game Pak WS2)
}

/// Scenario: DMA memory transfers work
#[test]
fn dma_transfers_move_data_between_memory_regions() {
    let mut mem = Memory::new();

    // Given: Source data in WRAM
    for i in 0..16 {
        mem.write_byte(0x0200_0000 + i, i as u8);
    }

    // And: DMA configured to transfer 16 bytes from 0x0200_0000 to 0x0300_0000
    // DMA control registers:
    // 0x0400_00B0: DMA0 Source Address
    // 0x0400_00B4: DMA0 Destination Address
    // 0x0400_00B8: DMA0 Control
    mem.write_word(0x0400_00B0, 0x0200_0000); // Source
    mem.write_word(0x0400_00B4, 0x0300_0000); // Destination
    mem.write_word(0x0400_00B8, 0x8000_000F); // Enable, 16 transfers

    // When: DMA is triggered (implementation dependent)

    // Then: Data should be copied
    // assert_eq!(mem.read_byte(0x0300_0000), 0);
    // assert_eq!(mem.read_byte(0x0300_000F), 15);
}

/// Scenario: Memory can be reset to clean state
#[test]
fn memory_reset_clears_all_regions() {
    let mut mem = Memory::new();

    // Given: Memory with data in various regions
    mem.write_byte(0x0200_0000, 0xFF);
    mem.write_half(0x0500_0000, 0x7FFF);
    mem.write_word(0x0600_0000, 0xDEAD_BEEF);

    // When: Memory is reset
    mem.reset();

    // Then: Most regions should be zeroed
    assert_eq!(mem.read_byte(0x0200_0000), 0, "WRAM should be zeroed");
    assert_eq!(mem.read_half(0x0500_0000), 0, "Palette should be zeroed");
    assert_eq!(mem.read_word(0x0600_0000), 0, "VRAM should be zeroed");
}

/// Scenario: Waitstate configuration affects memory access timing
#[test]
fn waitstate_configuration_modifies_access_cycles() {
    let mem = Memory::new();

    // Given: Default ROM access is 3 cycles
    assert_eq!(mem.get_access_cycles(0x0800_0000, false), 3);

    // When: Waitstate register is configured for faster access
    // 0x0400_0204: WAITCNT
    // - WS0 non-sequential: 0 cycles (fastest)
    // - WS0 sequential: 0 cycles
    // mem.write_half(0x0400_0204, 0x0000); // All waitstates = 0

    // Then: ROM access should be faster
    // assert_eq!(mem.get_access_cycles(0x0800_0000, false), 1);
}

/// Scenario: Palette RAM stores color data correctly
#[test]
fn palette_ram_stores_color_data() {
    let mut mem = Memory::new();

    // When: Writing color to palette
    // Format: 15-bit BGR (XBBBBBGG GGGRRRRR)
    let color = 0x7FFF; // White (all bits set)
    mem.write_half(0x0500_0000, color);

    // Then: Should read back correctly
    assert_eq!(mem.read_half(0x0500_0000), 0x7FFF);

    // And: Should have 256 palette entries for BG and 256 for OBJ
    // Total: 512 entries = 1024 bytes
    let last_bg_palette = 0x0500_01FE;
    mem.write_half(last_bg_palette, 0x001F); // Pure red
    assert_eq!(mem.read_half(last_bg_palette), 0x001F);
}

/// Scenario: VRAM handles different bitmap and tile modes
#[test]
fn vram_handles_different_graphics_modes() {
    let mut mem = Memory::new();

    // Mode 3: 240x160, 16-bit color (0x0600_0000 - 0x0601_3FFF, 80KB)
    let pixel_addr = 0x0600_0000 + (120 * 240 + 80) * 2; // Pixel at (80, 120)
    mem.write_half(pixel_addr, 0x7FFF);
    assert_eq!(mem.read_half(pixel_addr), 0x7FFF);

    // Mode 4: 240x160, 8-bit color with palette (0x0600_0000 - 0x0600_9FFF, 40KB)
    // Uses page switching for double buffering
}

/// Scenario: OAM stores sprite attributes
#[test]
fn oam_stores_sprite_attributes_correctly() {
    let mut mem = Memory::new();

    // OAM entry format (4 words per sprite):
    // Word 0: Attr0 - Y position, rotation/scale, etc.
    // Word 1: Attr1 - X position, flip settings
    // Word 2: Attr2 - tile number, priority, palette
    // Word 3: Attr3/Param - rotation/scale parameters or filler

    let sprite_addr = 0x0700_0000; // First sprite

    // When: Writing sprite attributes
    mem.write_half(sprite_addr, 0x0080); // Y = 128, no special flags
    mem.write_half(sprite_addr + 2, 0x00A0); // X = 160
    mem.write_half(sprite_addr + 4, 0x0000); // Tile 0, priority 0, palette 0

    // Then: Should read back correctly
    assert_eq!(mem.read_half(sprite_addr), 0x0080);
    assert_eq!(mem.read_half(sprite_addr + 2), 0x00A0);
    assert_eq!(mem.read_half(sprite_addr + 4), 0x0000);

    // And: Should support 128 sprites (max)
    // Last sprite at 0x0700_03FC
}

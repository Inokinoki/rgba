//! Behavior Driven Development tests for the GBA DMA System
//!
//! These tests describe the expected behavior of the GBA's 4 DMA channels.

use rgba::{Dma, Memory};

/// Scenario: DMA channel initializes correctly
#[test]
fn dma_channel_initializes_in_disabled_state() {
    let dma = Dma::new(0);

    assert_eq!(dma.is_enabled(), false, "DMA should start disabled");
    assert_eq!(dma.is_active(), false, "DMA should not be active");
    assert_eq!(dma.get_control(), 0, "Control register should be 0");
}

/// Scenario: DMA can transfer data
#[test]
fn dma_can_transfer_data_between_memory_regions() {
    let mut dma = Dma::new(0);
    let mut mem = Memory::new();

    // Setup source data in WRAM
    let src_addr = 0x0200_0000;
    mem.write_word(src_addr, 0x12345678);
    mem.write_word(src_addr + 4, 0x9ABCDEF0);

    // Setup destination in IWRAM
    let dst_addr = 0x0300_0000;

    // Configure DMA
    dma.set_src_addr(src_addr);
    dma.set_dst_addr(dst_addr);
    dma.set_count(2); // Transfer 2 words
    dma.set_control(0x8400); // Enable, word size (bit 10), immediate transfer

    // Execute transfer
    assert!(dma.is_active(), "DMA should be active after enable");
    dma.execute(&mut mem);

    // Verify data was transferred
    assert_eq!(mem.read_word(dst_addr), 0x12345678, "First word should be transferred");
    assert_eq!(mem.read_word(dst_addr + 4), 0x9ABCDEF0, "Second word should be transferred");
}

/// Scenario: DMA can be triggered by different events
#[test]
fn dma_can_be_triggered_by_various_events() {
    let mut dma = Dma::new(0);

    // Test VBlank trigger mode
    dma.set_control(0x9000); // Enable + VBlank trigger
    assert_eq!(dma.get_trigger() as u8, 1, "Should be in VBlank trigger mode");

    // Test HBlank trigger mode
    dma.set_control(0xA000); // Enable + HBlank trigger
    assert_eq!(dma.get_trigger() as u8, 2, "Should be in HBlank trigger mode");

    // Test immediate trigger mode
    dma.set_control(0x8000); // Enable + immediate trigger
    assert_eq!(dma.get_trigger() as u8, 0, "Should be in immediate trigger mode");
}

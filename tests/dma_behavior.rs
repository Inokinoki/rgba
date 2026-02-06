//! Behavior Driven Development tests for the GBA DMA System
//!
//! These tests describe the expected behavior of the GBA's 4 DMA channels.

use rgba::Dma;

/// Scenario: DMA channel initializes correctly
#[test]
fn dma_channel_initializes_in_disabled_state() {
    let dma = Dma::new(0);

    // TODO: Test initial state
}

/// Scenario: DMA can transfer data
#[test]
fn dma_can_transfer_data_between_memory_regions() {
    let mut dma = Dma::new(0);

    // TODO: Test data transfer
}

/// Scenario: DMA can be triggered by different events
#[test]
fn dma_can_be_triggered_by_various_events() {
    let mut dma = Dma::new(0);

    // Immediate, VBlank, HBlank, etc.
}

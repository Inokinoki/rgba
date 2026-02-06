#![cfg(test)]

// Behavior tests for CPU functionality
mod cpu_behavior;
// Behavior tests for Memory system
mod memory_behavior;
// Behavior tests for PPU (graphics)
mod ppu_behavior;
// Behavior tests for APU (audio)
mod apu_behavior;
// Behavior tests for Timer system
mod timer_behavior;
// Behavior tests for DMA
mod dma_behavior;
// Behavior tests for Input handling
mod input_behavior;
// Integration tests
mod integration;

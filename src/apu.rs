//! GBA Audio Processing Unit (APU)
//!
//! Handles sound generation including:
//! - 4 PSG channels (2 square wave, 1 wave, 1 noise)
//! - Direct Sound A/B (sample playback)
//! - FIFO DMA for audio streaming

/// GBA Audio Processing Unit
pub struct Apu {
    // PSG channels
    // Channel 1-2: Square wave with sweep and envelope
    // Channel 3: Wave sample playback
    // Channel 4: Noise generator
    enabled: bool,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            enabled: false,
        }
    }

    pub fn reset(&mut self) {
        self.enabled = false;
    }

    /// Step the APU forward by given number of cycles
    pub fn step(&mut self, _cycles: u32) {
        // TODO: Implement audio generation
    }
}

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}

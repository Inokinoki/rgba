//! GBA Direct Memory Access Controller
//!
//! The GBA has 4 DMA channels that can transfer data between
//! memory regions without CPU intervention.

/// GBA DMA Channel
pub struct Dma {
    num: u8,
    src_addr: u32,
    dst_addr: u32,
    count: u16,
    control: u16,
    enabled: bool,
}

impl Dma {
    pub fn new(num: u8) -> Self {
        Self {
            num,
            src_addr: 0,
            dst_addr: 0,
            count: 0,
            control: 0,
            enabled: false,
        }
    }

    pub fn reset(&mut self) {
        self.src_addr = 0;
        self.dst_addr = 0;
        self.count = 0;
        self.control = 0;
        self.enabled = false;
    }
}

impl Default for Dma {
    fn default() -> Self {
        Self::new(0)
    }
}

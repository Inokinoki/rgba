//! GBA Timers
//!
//! The GBA has 4 timers that can operate independently or cascade.
//! Each timer can trigger DMA or interrupts on overflow.

/// GBA Timer
pub struct Timer {
    num: u8,
    counter: u16,
    reload: u16,
    control: u16,
    prescaler_shift: u8,
    enabled: bool,
    count_up: bool,
    irq: bool,
}

impl Timer {
    pub fn new(num: u8) -> Self {
        Self {
            num,
            counter: 0,
            reload: 0,
            control: 0,
            prescaler_shift: 0,
            enabled: false,
            count_up: false,
            irq: false,
        }
    }

    pub fn reset(&mut self) {
        self.counter = 0;
        self.reload = 0;
        self.control = 0;
        self.enabled = false;
        self.count_up = false;
        self.irq = false;
        self.prescaler_shift = 0;
    }

    /// Step the timer forward by given number of cycles
    pub fn step(&mut self, _cycles: u32) {
        // TODO: Implement timer counting
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new(0)
    }
}

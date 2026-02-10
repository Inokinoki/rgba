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
    overflow_pending: bool,
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
            overflow_pending: false,
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
        self.overflow_pending = false;
    }

    /// Step the timer forward by given number of cycles
    pub fn step(&mut self, cycles: u32) {
        if !self.enabled {
            return;
        }

        if self.count_up && self.num > 0 {
            // Count-up timing: only increment when previous timer overflows
            // This is handled by the Gba struct calling trigger_count_up
            return;
        }

        // Apply prescaler
        let prescaler = 1 << self.prescaler_shift;
        let scaled_cycles = cycles >> self.prescaler_shift;

        if scaled_cycles == 0 {
            return;
        }

        // Check if overflow will occur
        let old_counter = self.counter;
        let (new_counter, overflow) = self.counter.overflowing_add(scaled_cycles as u16);

        if overflow || (new_counter < old_counter && scaled_cycles > 0) {
            // Timer overflowed
            self.counter = self.reload;
            let remaining = (scaled_cycles as u16).wrapping_sub(0xFFFF_u16.wrapping_sub(old_counter));
            self.counter = self.counter.wrapping_add(remaining);
            self.overflow_pending = true;
        } else {
            self.counter = new_counter;
        }
    }

    /// Trigger count-up timing (called when previous timer overflows)
    pub fn trigger_count_up(&mut self) {
        if !self.enabled || !self.count_up {
            return;
        }

        let (new_counter, overflow) = self.counter.overflowing_add(1);
        if overflow {
            self.counter = self.reload;
            self.overflow_pending = true;
        } else {
            self.counter = new_counter;
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn get_control(&self) -> u16 {
        self.control
    }

    pub fn set_control(&mut self, value: u16) {
        self.control = value;
        self.prescaler_shift = match value & 0x03 {
            0 => 0,  // 1 cycle
            1 => 6,  // 64 cycles
            2 => 8,  // 256 cycles
            3 => 10, // 1024 cycles
            _ => 0,
        };
        self.count_up = (value & 0x04) != 0;
        self.irq = (value & 0x40) != 0;
        self.enabled = (value & 0x80) != 0;
    }

    pub fn get_reload(&self) -> u16 {
        self.reload
    }

    pub fn set_reload(&mut self, value: u16) {
        self.reload = value;
    }

    pub fn get_counter(&self) -> u16 {
        self.counter
    }

    pub fn is_irq_enabled(&self) -> bool {
        self.irq
    }

    pub fn is_count_up(&self) -> bool {
        self.count_up
    }

    pub fn did_overflow(&mut self) -> bool {
        let result = self.overflow_pending;
        self.overflow_pending = false;
        result
    }

    pub fn set_overflow(&mut self) {
        self.overflow_pending = true;
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new(0)
    }
}

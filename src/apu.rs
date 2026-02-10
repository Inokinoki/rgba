//! GBA Audio Processing Unit (APU)
//!
//! Handles sound generation including:
//! - 4 PSG channels (2 square wave, 1 wave, 1 noise)
//! - Direct Sound A/B (sample playback)
//! - FIFO DMA for audio streaming

/// PSG Square Wave Channel (Channel 1-2)
#[derive(Debug)]
pub struct SquareChannel {
    enabled: bool,
    duty_cycle: u8,      // 0-3 (12.5%, 25%, 50%, 75%)
    length_enabled: bool,
    length_load: u8,     // 0-63
    length_counter: u8,
    envelope_enabled: bool,
    envelope_direction: bool, // true = increasing, false = decreasing
    envelope_step: u8,   // 0-7
    envelope_volume: u8, // 0-15
    envelope_counter: u8,
    sweep_enabled: bool, // Channel 1 only
    sweep_shift: u8,     // 0-7
    sweep_direction: bool, // true = addition, false = subtraction
    sweep_time: u8,      // 0-7
    sweep_counter: u8,
    frequency: u16,      // 0-2047
    frequency_counter: u16,
    duty_position: u8,
    output_volume: u8,   // 0-15
}

impl SquareChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            duty_cycle: 0,
            length_enabled: false,
            length_load: 0,
            length_counter: 0,
            envelope_enabled: false,
            envelope_direction: false,
            envelope_step: 0,
            envelope_volume: 0,
            envelope_counter: 0,
            sweep_enabled: false,
            sweep_shift: 0,
            sweep_direction: false,
            sweep_time: 0,
            sweep_counter: 0,
            frequency: 0,
            frequency_counter: 0,
            duty_position: 0,
            output_volume: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn step(&mut self, cycles: u32) {
        if !self.enabled {
            return;
        }

        // Frequency timer (one cycle = 4 cycles)
        let freq_period = (2048 - self.frequency) as u32 * 4;
        if cycles >= freq_period {
            self.duty_position = (self.duty_position + 1) % 8;
        }

        // Update output based on duty cycle and position
        self.output_volume = if self.get_duty_output() {
            self.envelope_volume
        } else {
            0
        };
    }

    fn get_duty_output(&self) -> bool {
        match self.duty_cycle {
            0 => matches!(self.duty_position, 0..=0),   // 12.5%
            1 => matches!(self.duty_position, 0..=1),   // 25%
            2 => matches!(self.duty_position, 0..=3),   // 50%
            3 => matches!(self.duty_position, 0..=5),   // 75%
            _ => false,
        }
    }

    pub fn get_output(&self) -> u8 {
        self.output_volume
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_frequency(&mut self, freq: u16) {
        self.frequency = freq & 0x7FF;
    }

    pub fn set_duty_cycle(&mut self, duty: u8) {
        self.duty_cycle = duty & 0x3;
    }

    pub fn trigger(&mut self) {
        self.enabled = true;
        self.length_counter = 64 - self.length_load as u8;
        self.envelope_counter = self.envelope_step;
        self.frequency_counter = 0;
    }
}

/// PSG Wave Channel (Channel 3)
#[derive(Debug)]
pub struct WaveChannel {
    enabled: bool,
    length_enabled: bool,
    length_load: u8,     // 0-255
    length_counter: u8,
    volume_code: u8,     // 0-3
    frequency: u16,      // 0-2047
    frequency_counter: u16,
    wave_position: u8,
    wave_ram: [u8; 32],  // 32 4-bit samples
    output_volume: u8,
}

impl WaveChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            length_enabled: false,
            length_load: 0,
            length_counter: 0,
            volume_code: 0,
            frequency: 0,
            frequency_counter: 0,
            wave_position: 0,
            wave_ram: [0; 32],
            output_volume: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn step(&mut self, cycles: u32) {
        if !self.enabled {
            return;
        }

        // Frequency timer (one cycle = 2 cycles)
        let freq_period = (2048 - self.frequency) as u32 * 2;
        if cycles >= freq_period {
            self.wave_position = (self.wave_position + 1) % 32;
        }

        // Get sample and apply volume
        let idx = (self.wave_position / 2) as usize;
        let sample = self.wave_ram[idx];
        let nibble = if self.wave_position % 2 == 0 {
            sample >> 4
        } else {
            sample & 0xF
        };

        self.output_volume = match self.volume_code {
            0 => 0,
            1 => nibble,
            2 => nibble >> 1,
            3 => nibble >> 2,
            _ => 0,
        };
    }

    pub fn get_output(&self) -> u8 {
        self.output_volume
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_frequency(&mut self, freq: u16) {
        self.frequency = freq & 0x7FF;
    }

    pub fn set_wave_ram(&mut self, index: usize, value: u8) {
        if index < 32 {
            self.wave_ram[index] = value;
        }
    }
}

/// PSG Noise Channel (Channel 4)
#[derive(Debug)]
pub struct NoiseChannel {
    enabled: bool,
    length_enabled: bool,
    length_load: u8,
    length_counter: u8,
    envelope_enabled: bool,
    envelope_direction: bool,
    envelope_step: u8,
    envelope_volume: u8,
    envelope_counter: u8,
    clock_shift: u8,     // 0-14
    width_mode: bool,     // false = 7-bit, true = 15-bit
    lfsr: u16,
    output_volume: u8,
}

impl NoiseChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            length_enabled: false,
            length_load: 0,
            length_counter: 0,
            envelope_enabled: false,
            envelope_direction: false,
            envelope_step: 0,
            envelope_volume: 0,
            envelope_counter: 0,
            clock_shift: 0,
            width_mode: false,
            lfsr: 0x7FFF,
            output_volume: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn step(&mut self, cycles: u32) {
        if !self.enabled {
            return;
        }

        // Noise generation with LFSR
        let divisor = 1 << (self.clock_shift.min(14));
        if cycles >= divisor as u32 {
            // XOR bit 0 and bit 1
            let xor = ((self.lfsr & 1) ^ ((self.lfsr >> 1) & 1)) != 0;
            self.lfsr = (self.lfsr >> 1) | ((xor as u16) << 14);

            if !self.width_mode {
                // 7-bit mode: clear bit 6
                self.lfsr &= !0x40;
            }

            self.output_volume = if (self.lfsr & 1) == 0 {
                self.envelope_volume
            } else {
                0
            };
        }
    }

    pub fn get_output(&self) -> u8 {
        self.output_volume
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Direct Sound Channel (FIFO DMA)
#[derive(Debug)]
pub struct DirectSoundChannel {
    enabled: bool,
    fifo: [u8; 32],     // 8 words * 4 bytes
    fifo_read: u8,
    fifo_write: u8,
    fifo_count: u8,
    volume: u8,         // 0-3 (50%, 100%, 25%, 50% with right shift)
    timer: u8,          // Timer 0 or 1
    output_right: bool,
    output_left: bool,
    current_sample: i16,
}

impl DirectSoundChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            fifo: [0; 32],
            fifo_read: 0,
            fifo_write: 0,
            fifo_count: 0,
            volume: 0,
            timer: 0,
            output_right: false,
            output_left: false,
            current_sample: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn write_fifo(&mut self, data: u32) {
        // Write 4 bytes (one word) to FIFO
        for i in 0..4 {
            self.fifo[self.fifo_write as usize] = ((data >> (i * 8)) & 0xFF) as u8;
            self.fifo_write = (self.fifo_write + 1) % 32;
            if self.fifo_count < 8 {
                self.fifo_count += 1;
            }
        }
    }

    pub fn read_sample(&mut self) -> i16 {
        if self.fifo_count == 0 {
            return 0;
        }

        // Read 2 bytes (one halfword) from FIFO
        let low = self.fifo[self.fifo_read as usize] as i16;
        self.fifo_read = (self.fifo_read + 1) % 32;
        let high = self.fifo[self.fifo_read as usize] as i16;
        self.fifo_read = (self.fifo_read + 1) % 32;
        self.fifo_count -= 2;

        let sample = (high << 8) | low;
        self.current_sample = sample;

        sample
    }

    pub fn get_output(&self) -> i16 {
        let volume_shift = match self.volume {
            0 => 1,  // 50%
            1 => 0,  // 100%
            2 => 2,  // 25%
            3 => 1,  // 50% with right shift
            _ => 1,
        };
        self.current_sample >> volume_shift
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// GBA Audio Processing Unit
pub struct Apu {
    // PSG channels
    square1: SquareChannel,
    square2: SquareChannel,
    wave: WaveChannel,
    noise: NoiseChannel,

    // Direct Sound channels
    ds_a: DirectSoundChannel,
    ds_b: DirectSoundChannel,

    // Master control
    master_enabled: bool,
    volume_left: u8,    // 0-7
    volume_right: u8,   // 0-7

    // Mixing
    left_enabled: [bool; 8],  // Enable each channel on left
    right_enabled: [bool; 8], // Enable each channel on right

    // Output
    output_left: i16,
    output_right: i16,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            square1: SquareChannel::new(),
            square2: SquareChannel::new(),
            wave: WaveChannel::new(),
            noise: NoiseChannel::new(),
            ds_a: DirectSoundChannel::new(),
            ds_b: DirectSoundChannel::new(),
            master_enabled: false,
            volume_left: 0,
            volume_right: 0,
            left_enabled: [false; 8],
            right_enabled: [false; 8],
            output_left: 0,
            output_right: 0,
        }
    }

    pub fn reset(&mut self) {
        self.square1.reset();
        self.square2.reset();
        self.wave.reset();
        self.noise.reset();
        self.ds_a.reset();
        self.ds_b.reset();
        self.master_enabled = false;
        self.volume_left = 0;
        self.volume_right = 0;
        self.left_enabled = [false; 8];
        self.right_enabled = [false; 8];
        self.output_left = 0;
        self.output_right = 0;
    }

    /// Step the APU forward by given number of cycles
    pub fn step(&mut self, cycles: u32) {
        if !self.master_enabled {
            return;
        }

        // Step PSG channels
        self.square1.step(cycles);
        self.square2.step(cycles);
        self.wave.step(cycles);
        self.noise.step(cycles);

        // Mix all channels
        let mut left_mixed = 0i32;
        let mut right_mixed = 0i32;

        // PSG channels
        if self.left_enabled[0] { left_mixed += self.square1.get_output() as i32; }
        if self.right_enabled[0] { right_mixed += self.square1.get_output() as i32; }
        if self.left_enabled[1] { left_mixed += self.square2.get_output() as i32; }
        if self.right_enabled[1] { right_mixed += self.square2.get_output() as i32; }
        if self.left_enabled[2] { left_mixed += self.wave.get_output() as i32; }
        if self.right_enabled[2] { right_mixed += self.wave.get_output() as i32; }
        if self.left_enabled[3] { left_mixed += self.noise.get_output() as i32; }
        if self.right_enabled[3] { right_mixed += self.noise.get_output() as i32; }

        // Direct Sound channels
        if self.left_enabled[4] { left_mixed += self.ds_a.get_output() as i32; }
        if self.right_enabled[4] { right_mixed += self.ds_a.get_output() as i32; }
        if self.left_enabled[5] { left_mixed += self.ds_b.get_output() as i32; }
        if self.right_enabled[5] { right_mixed += self.ds_b.get_output() as i32; }

        // Apply master volume
        self.output_left = ((left_mixed * self.volume_left as i32) / 7) as i16;
        self.output_right = ((right_mixed * self.volume_right as i32) / 7) as i16;
    }

    pub fn get_output_left(&self) -> i16 {
        self.output_left
    }

    pub fn get_output_right(&self) -> i16 {
        self.output_right
    }

    pub fn set_master_enabled(&mut self, enabled: bool) {
        self.master_enabled = enabled;
    }

    pub fn is_master_enabled(&self) -> bool {
        self.master_enabled
    }

    pub fn set_volume_left(&mut self, volume: u8) {
        self.volume_left = volume & 0x7;
    }

    pub fn set_volume_right(&mut self, volume: u8) {
        self.volume_right = volume & 0x7;
    }

    pub fn set_channel_enabled_left(&mut self, channel: usize, enabled: bool) {
        if channel < 8 {
            self.left_enabled[channel] = enabled;
        }
    }

    pub fn set_channel_enabled_right(&mut self, channel: usize, enabled: bool) {
        if channel < 8 {
            self.right_enabled[channel] = enabled;
        }
    }

    // PSG channel access
    pub fn get_square1(&mut self) -> &mut SquareChannel {
        &mut self.square1
    }

    pub fn get_square2(&mut self) -> &mut SquareChannel {
        &mut self.square2
    }

    pub fn get_wave(&mut self) -> &mut WaveChannel {
        &mut self.wave
    }

    pub fn get_noise(&mut self) -> &mut NoiseChannel {
        &mut self.noise
    }

    // Direct Sound access
    pub fn get_ds_a(&mut self) -> &mut DirectSoundChannel {
        &mut self.ds_a
    }

    pub fn get_ds_b(&mut self) -> &mut DirectSoundChannel {
        &mut self.ds_b
    }
}

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}

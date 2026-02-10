mod cpu;
mod mem;
mod ppu;
mod apu;
mod timer;
mod dma;
mod input;

pub use cpu::Cpu;
pub use mem::Memory;
pub use ppu::Ppu;
pub use apu::Apu;
pub use timer::Timer;
pub use dma::Dma;
pub use input::{Input, KeyState};

use std::fmt;

/// Represents the GBA console
pub struct Gba {
    pub cpu: Cpu,
    pub mem: Memory,
    pub ppu: Ppu,
    pub apu: Apu,
    pub timers: [Timer; 4],
    pub dma: [Dma; 4],
    pub input: Input,
}

impl Gba {
    /// Creates a new GBA instance
    pub fn new() -> Self {
        let mut gba = Self {
            cpu: Cpu::new(),
            mem: Memory::new(),
            ppu: Ppu::new(),
            apu: Apu::new(),
            timers: [
                Timer::new(0),
                Timer::new(1),
                Timer::new(2),
                Timer::new(3),
            ],
            dma: [
                Dma::new(0),
                Dma::new(1),
                Dma::new(2),
                Dma::new(3),
            ],
            input: Input::new(),
        };
        gba.cpu.reset(); // Initialize CPU to proper GBA state
        gba
    }

    /// Resets the GBA to its initial state
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.mem.reset();
        self.ppu.reset();
        self.apu.reset();
        for timer in &mut self.timers {
            timer.reset();
        }
        for dma in &mut self.dma {
            dma.reset();
        }
        self.input.reset();
    }

    /// Runs the emulator for one frame
    pub fn run_frame(&mut self) {
        // GBA runs at ~16.78 MHz
        // Each frame is 280896 cycles (59.57 Hz)
        for _ in 0..280896 {
            self.step();
        }
    }

    /// Executes a single step
    pub fn step(&mut self) {
        let cycles = self.cpu.step(&mut self.mem);
        self.ppu.step(cycles);
        self.apu.step(cycles);
        for timer in &mut self.timers {
            timer.step(cycles);
        }
    }

    /// Loads a ROM into memory
    pub fn load_rom(&mut self, data: Vec<u8>) {
        self.mem.load_rom(data);
    }

    /// Loads a ROM from a file path
    pub fn load_rom_path(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;
        use std::io::Read;

        let mut file = fs::File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        self.load_rom(data);
        Ok(())
    }

    /// Get a reference to the PPU
    pub fn ppu(&self) -> &Ppu {
        &self.ppu
    }

    /// Get a mutable reference to the input system
    pub fn input_mut(&mut self) -> &mut Input {
        &mut self.input
    }

    /// Get a reference to the CPU
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }
}

impl Default for Gba {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Gba {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Gba")
            .field("cpu", &self.cpu)
            .finish()
    }
}

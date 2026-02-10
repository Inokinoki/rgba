# RGBA GBA Emulator - Quick Start Guide

## What is RGBA?

RGBA is a **Game Boy Advance emulator** written in Rust using Behavior Driven Development principles. It accurately emulates the GBA hardware including:

- ARM7TDMI CPU (ARM and Thumb modes)
- Complete memory system
- Graphics processing (all 6 display modes)
- Audio (PSG + Direct Sound)
- Timers and DMA
- Input handling

## Quick Start

### 1. Clone and Build

```bash
# Clone the repository
git clone https://github.com/yourusername/rgba.git
cd rgba

# Build the library
cargo build

# Run tests
cargo test
```

### 2. Using as a Library

```rust
use rgba::Gba;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create emulator
    let mut gba = Gba::new();

    // Load a ROM
    gba.load_rom("path/to/rom.gba")?;

    // Run for one frame (approx 280000 cycles)
    for _ in 0..2800 {
        gba.step();
    }

    // Access components
    let cpu = gba.cpu();
    let ppu = gba.ppu();
    let input = gba.input_mut();

    Ok(())
}
```

### 3. Running the GUI

```bash
# Run with a ROM
cargo run --example gui_emulator --features gui -- path/to/rom.gba

# Run without ROM (shows test pattern)
cargo run --example gui_emulator --features gui
```

**GUI Controls:**
- Arrow Keys: D-Pad
- Z: A button
- X: B button
- Enter: Start
- Right Shift: Select
- A/S: L/R shoulder buttons
- P: Pause/Resume
- R: Reset
- Q: Quit

## Examples

The project includes several examples demonstrating different features:

```bash
# Basic usage
cargo run --example quick_start

# CPU testing
cargo run --example cpu_test

# Memory operations
cargo run --example memory_test

# Input handling
cargo run --example input_demo

# Graphics demo
cargo run --example graphics_demo

# GUI emulator
cargo run --example gui_emulator --features gui -- rom.gba
```

## Running Tests

RGBA uses Behavior Driven Development with comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test cpu_behavior
cargo test --test timer_behavior
cargo test --test dma_behavior

# Run with output
cargo test -- --nocapture

# Run tests in parallel
cargo test -- --test-threads=4
```

## Test Coverage

- **62 tests** covering all major components
- **100% pass rate**
- Tests for:
  - CPU instructions (ARM + Thumb)
  - Memory operations
  - Graphics rendering
  - Audio generation
  - Timer functionality
  - DMA transfers
  - Input handling

## Architecture

### Components

1. **CPU** (`src/cpu.rs`)
   - ARM7TDMI processor
   - ARM and Thumb instruction sets
   - Pipeline simulation
   - Register banking

2. **Memory** (`src/mem.rs`)
   - Full GBA memory map
   - Proper timing simulation
   - ROM loading

3. **PPU** (`src/ppu.rs`)
   - Display modes 0-5
   - Background layers
   - Sprite rendering
   - Special effects

4. **APU** (`src/apu.rs`)
   - PSG channels (4)
   - Direct Sound (2)
   - Stereo mixing

5. **Timers** (`src/timer.rs`)
   - 4 independent timers
   - Cascading support
   - IRQ generation

6. **DMA** (`src/dma.rs`)
   - 4 DMA channels
   - Multiple trigger modes
   - Flexible addressing

7. **Input** (`src/input.rs`)
   - All GBA buttons
   - Keypad state

## Performance

The emulator targets **60 FPS** (the original GBA refresh rate):

- Single frame: ~280,000 CPU cycles
- Cycle-accurate timing
- Efficient Rust implementation

## Troubleshooting

### Build Errors

If you encounter build errors:

```bash
# Update Rust
rustup update

# Clean build
cargo clean
cargo build

# With verbose output
cargo build --verbose
```

### Test Failures

All tests should pass. If you see failures:

```bash
# Run tests individually
cargo test -- --test-threads=1

# Check for warnings
cargo test -- --show-output

# Update dependencies
cargo update
```

### GUI Issues

If the GUI won't run:

```bash
# Ensure GUI feature is enabled
cargo run --example gui_emulator --features gui

# Check graphics drivers
# Update your graphics drivers if needed

# Try software rendering
# (depends on your system)
```

## Documentation

- **Implementation Report:** `IMPLEMENTATION_REPORT.md`
- **GUI Guide:** `GUI_README.md`
- **API Documentation:** Run `cargo doc --open`

## Contributing

We welcome contributions! Please:

1. Fork the repository
2. Create a feature branch
3. Write tests for new features
4. Ensure all tests pass
5. Submit a pull request

## License

MIT License - See LICENSE file for details

## Acknowledgments

- GBA hardware documentation from GBATek
- Rust community for excellent tools
- Test-driven development community

## Support

- **Issues:** Report bugs on GitHub
- **Discussions:** Use GitHub Discussions
- **Documentation:** See inline docs and examples

---

**Happy Emulating! 🎮**

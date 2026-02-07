# RGBA GBA Emulator - API Documentation

Complete API reference for the RGBA Game Boy Advance emulator.

## Table of Contents

- [Quick Start](#quick-start)
- [Core Types](#core-types)
- [CPU API](#cpu-api)
- [Memory API](#memory-api)
- [PPU API](#ppu-api)
- [Input API](#input-api)
- [Full System API](#full-system-api)

## Quick Start

```rust
use rgba::Gba;

fn main() {
    // Create a new GBA instance
    let mut gba = Gba::new();

    // Load a ROM file
    let rom_data = std::fs::read("game.gba").expect("Failed to load ROM");
    gba.load_rom(rom_data);

    // Run one frame (59.57 Hz)
    gba.run_frame();

    // Check state
    println!("PC: 0x{:08X}", gba.cpu.get_pc());
    println!("Display Mode: {}", gba.ppu.get_display_mode());
}
```

## Core Types

### `Gba`

The main GBA console structure containing all components.

```rust
pub struct Gba {
    pub cpu: Cpu,
    pub mem: Memory,
    pub ppu: Ppu,
    pub apu: Apu,
    pub timers: [Timer; 4],
    pub dma: [Dma; 4],
    pub input: Input,
}
```

**Methods:**

- `new()` - Create a new GBA instance
- `reset(&mut self)` - Reset all components to initial state
- `load_rom(&mut self, data: Vec<u8>)` - Load ROM into memory
- `run_frame(&mut self)` - Execute one frame (280,896 cycles)
- `step(&mut self)` - Execute one CPU step

## CPU API

### `Cpu`

ARM7TDMI processor implementation.

**Creation & Reset:**
```rust
let mut cpu = Cpu::new();
cpu.reset();
```

**Register Access:**
```rust
cpu.set_reg(0, 0xDEADBEEF);      // Set R0
let val = cpu.get_reg(0);         // Get R0

cpu.set_sp(0x03007F00);         // Set stack pointer
let pc = cpu.get_pc();           // Get program counter
cpu.set_pc(0x08000000);        // Set program counter
```

**Processor Mode:**
```rust
use rgba::cpu::Mode;

cpu.set_mode(Mode::Irq);
assert_eq!(cpu.get_mode(), Mode::Irq);
```

**Flags:**
```rust
cpu.set_flag_c(true);           // Set carry flag
let carry = cpu.get_flag_c();  // Check carry flag

cpu.set_thumb_mode(true);       // Switch to Thumb mode
let is_thumb = cpu.is_thumb_mode();
```

**Execution:**
```rust
let cycles = cpu.step(&mut memory); // Execute one instruction
```

## Memory API

### `Memory`

Complete GBA memory map implementation.

**Creation & Reset:**
```rust
let mut mem = Memory::new();
mem.reset();
```

**Byte/Halfword/Word Access:**
```rust
// Byte (8-bit)
mem.write_byte(0x02000000, 0xAB);
let byte_val = mem.read_byte(0x02000000);

// Halfword (16-bit)
mem.write_half(0x05000000, 0x7FFF); // White color
let halfword = mem.read_half(0x05000000);

// Word (32-bit)
mem.write_word(0x06000000, 0xDEADBEEF);
let word = mem.read_word(0x06000000);
```

**ROM Loading:**
```rust
let rom: Vec<u8> = std::fs::read("game.gba").unwrap();
mem.load_rom(rom);
```

**Access Timing:**
```rust
// Get cycles for memory access
let cycles = mem.get_access_cycles(0x02000000, false);
// Returns: 3 for WRAM, 1 for IWRAM, etc.
```

## PPU API

### `Ppu`

Picture Processing Unit for graphics rendering.

**Display Control:**
```rust
ppu.set_display_enabled(true);
ppu.set_display_mode(3); // 240x160 16-bit bitmap

assert_eq!(ppu.get_width(), 240);
assert_eq!(ppu.get_height(), 160);
```

**Background Control:**
```rust
ppu.set_bg_enabled(0, true);   // Enable BG0
ppu.set_bg_priority(0, 2); // Set priority

assert!(ppu.is_bg_enabled(0));
```

**Pixel Access (Mode 3 - 16-bit bitmap):**
```rust
// Set pixel at (x, y) to RGB color
ppu.set_pixel_mode3(120, 80, 0x7FFF); // White

// Get pixel color
let color = ppu.get_pixel_mode3(120, 80);
```

**Pixel Access (Mode 4 - 8-bit paletted):**
```rust
// Set palette index
ppu.set_pixel_mode4(100, 50, 123); // (x, y, index)

// Get palette index
let index = ppu.get_pixel_mode4(100, 50);
```

**Sprite Control:**
```rust
ppu.set_sprite_enabled(0, true);
ppu.set_sprite_x(0, 120);
ppu.set_sprite_y(0, 80);
ppu.set_sprite_tile(0, 5);
ppu.set_sprite_priority(0, 2);
```

**Timing:**
```rust
ppu.step(1232); // Advance by one scanline

let vcount = ppu.get_vcount();
let in_vblank = ppu.is_in_vblank();
let in_hblank = ppu.is_in_hblank();
```

## Input API

### `Input`

Keypad input handling with active-low logic.

**Creation:**
```rust
let mut input = Input::new();
```

**Key State:**
```rust
use rgba::KeyState;

input.press_key(KeyState::A);
input.press_key(KeyState::B);
input.press_key(KeyState::START);

let is_a_pressed = input.is_key_pressed(KeyState::A);
assert!(is_a_pressed);

input.release_key(KeyState::A);
```

**Register Access:**
```rust
let keyreg = input.get_key_register();
// Returns 16-bit value with active-low bit pattern
// Bits 0-9: Key states (0 = pressed, 1 = released)
// Bits 10-15: Always set to 1
```

## Full System API

### `Gba` Methods

**Creation:**
```rust
let gba = Gba::new();
// Or with default:
let gba = Gba::default();
```

**System Control:**
```rust
gba.reset();           // Reset all components
gba.load_rom(data);  // Load ROM
gba.run_frame();     // Execute one frame
gba.step();          // Execute one CPU step
```

**Component Access:**
```rust
// Access individual components
gba.cpu.set_pc(0x08000000);
gba.mem.write_byte(0x02000000, 0xAB);
gba.ppu.set_display_enabled(true);
gba.input.press_key(rgba::KeyState::A);
```

## Usage Patterns

### Running a ROM

```rust
use rgba::Gba;
use std::fs;

fn main() {
    let mut gba = Gba::new();

    // Load ROM
    let rom = fs::read("demo.gba").expect("Failed to load ROM");
    gba.load_rom(rom);

    // Run for 60 frames (approximately 1 second)
    for _ in 0..60 {
        gba.run_frame();
    }

    println!("Executed 60 frames successfully!");
}
```

### Testing Memory Access

```rust
use rgba::Memory;

#[test]
fn test_memory_write_read() {
    let mut mem = Memory::new();

    // Write to WRAM
    mem.write_word(0x02000000, 0x12345678);

    // Read back
    let value = mem.read_word(0x02000000);
    assert_eq!(value, 0x12345678);
}
```

### Graphics Demo

```rust
use rgba::Gba;

fn draw_gradient(gba: &mut Gba) {
    gba.ppu.set_display_mode(3);
    gba.ppu.set_display_enabled(true);

    // Draw a simple gradient
    for y in 0..160 {
        for x in 0..240 {
            let color = ((x as u32) << 5) | (y as u32);
            gba.ppu.set_pixel_mode3(x, y, (color & 0x7FFF) as u16);
        }
    }
}
```

### Input Handling

```rust
use rgba::{Gba, KeyState};

fn handle_input(gba: &mut Gba) {
    // Simulate pressing A button
    gba.input.press_key(KeyState::A);

    // Check if A is pressed
    if gba.input.is_key_pressed(KeyState::A) {
        // Do something
        println!("A button pressed!");
    }

    // Release button
    gba.input.release_key(KeyState::A);
}
```

## Implementation Details

### ARM Instruction Encoding

The CPU implements ARM mode instructions following this format:

```
|31-28 | 27-26 | 25 | 24-21 | 20 | 19-16 | 15-12 | 11-0 |
| Cond |  Cat  | L | Op   | S | Rn    | Rd    | Operand2 |
```

- **Cond**: Condition code (0xE = Always)
- **Cat**: Category (00=DataProc, 01=LoadStore, 10=Branch, 11=Branch)
- **L**: Link flag for BL instruction
- **S**: Set flags flag
- **Rn**: Source register
- **Rd**: Destination register
- **Operand2**: Second operand (immediate or register with shift)

### Memory Map

```
0x0000_0000 - 0x0000_3FFF: BIOS (16KB)
0x0200_0000 - 0x0203_FFFF: WRAM (256KB)
0x0300_0000 - 0x0300_7FFF: IWRAM (32KB)
0x0400_0000 - 0x0400_03FF: IO Registers (1KB)
0x0500_0000 - 0x0500_03FF: Palette RAM (1KB)
0x0600_0000 - 0x0601_7FFF: VRAM (96KB)
0x0700_0000 - 0x0700_03FF: OAM (1KB)
0x0800_0000 - 0x0DFF_FFFF: ROM (max 32MB)
```

### Display Modes

- **Mode 0**: Tile/text mode - 4 background layers
- **Mode 1**: Tile/text mode - 3 BGs + 1 affine BG
- **Mode 2**: Tile/text mode - 2 affine BGs
- **Mode 3**: 240x160, 16-bit color bitmap
- **Mode 4**: 240x160, 8-bit palette + page switching
- **Mode 5**: 160x128, 16-bit color + page switching

### Best Practices

1. **Always load ROM before execution**
   ```rust
   mem.load_rom(rom_data);
   cpu.set_pc(0x08000000);
   ```

2. **Use little-endian for ROM data**
   ```rust
   let insn = 0xE0900001u32.to_le_bytes();
   rom[0..4].copy_from_slice(&insn);
   ```

3. **Check display mode before graphics operations**
   ```rust
   if gba.ppu.get_display_mode() == 3 {
       // Use mode 3 operations
   }
   ```

4. **Handle active-low input correctly**
   ```rust
   // Keys are active-low (0 = pressed, 1 = released)
   if !input.is_key_pressed(KeyState::A) {
       // A is pressed
   }
   ```

5. **Reset system between ROM loads**
   ```rust
   gba.reset();
   gba.load_rom(new_rom);
   ```

## See Also

- [README.md](README.md) - Project overview
- [FINAL_ACHIEVEMENT.md](FINAL_ACHIEVEMENT.md) - Development journey
- [ITERATION_SUMMARY.md](ITERATION_SUMMARY.md) - Technical details

---

**Version**: 0.1.0
**Status**: Complete - All 62 tests passing
**Language**: Rust 2021 Edition
**License**: MIT

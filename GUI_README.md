# RGBA GBA Emulator - GUI Application

## Running the GUI Emulator

The RGBA emulator includes an optional GUI application for running Game Boy Advance ROMs with a graphical interface.

### Quick Start

```bash
# Run with a ROM file
cargo run --example gui_emulator --features gui -- path/to/rom.gba

# Run without ROM (shows test pattern)
cargo run --example gui_emulator --features gui
```

### Controls

| Keyboard | GBA Button | Description |
|----------|------------|-------------|
| Arrow Keys | D-Pad | Directional control |
| Z | A | A button |
| X | B | B button |
| Enter | Start | Start button |
| Right Shift | Select | Select button |
| A | L | Left shoulder button |
| S | R | Right shoulder button |
| R | - | Reset emulator |
| Escape | - | Quit emulator |

### Features

- **Real-time emulation** at 60 FPS
- **240x160 resolution** (scaled 3x = 720x480 window)
- **FPS counter** displayed in window title
- **Display mode support:**
  - Mode 3: 16-bit bitmap (RGB565)
  - Mode 4: 8-bit paletted bitmap
  - Test pattern for unsupported modes
- **Keyboard input** mapped to GBA buttons
- **Reset functionality**

### Display Modes

The emulator renders different modes as follows:

- **Mode 3:** 16-bit bitmap with proper RGB565 to RGB888 conversion
- **Mode 4:** 8-bit paletted (grayscale for simplicity)
- **Modes 0, 1, 2, 5:** Test pattern (gradient display)

### Requirements

The GUI uses `minifb` which requires:
- Linux: X11 libraries
- Windows: No additional requirements
- macOS: No additional requirements

### Building

```bash
# Build the GUI
cargo build --example gui_emulator --features gui

# Run optimized build
cargo build --example gui_emulator --features gui --release
```

### Performance

The emulator targets exactly 60 FPS (the original GBA refresh rate) using frame rate limiting.

### Troubleshooting

**Window won't open:**
- Ensure you have X11 libraries installed on Linux: `sudo apt-get install libx11-dev`
- Try running without a ROM first to see the test pattern

**Poor performance:**
- Use release build: `cargo run --example gui_emulator --features gui --release`
- Close other applications

**Display issues:**
- The emulator shows a test pattern when no ROM is loaded or in unsupported display modes
- Ensure your ROM is a valid GBA ROM file

### Technical Details

- **Library:** Uses `minifb` for cross-platform windowing
- **Rendering:** Direct pixel buffer manipulation
- **Timing:** Frame-accurate emulation with ~2800 CPU steps per frame
- **Input:** Polling-based keyboard input handling

### Example Code

```rust
use rgba::Gba;

// Create emulator
let mut gba = Gba::new();

// Load ROM
gba.load_rom_path("rom.gba")?;

// Run one frame
for _ in 0..2800 {
    gba.step();
}

// Access display
let ppu = gba.ppu();
let mode = ppu.get_display_mode();
```

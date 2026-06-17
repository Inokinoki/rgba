# RGBA — Game Boy Advance Emulator

A GBA emulator written in Rust, focused on correctness and readability.

## Features

- **ARM7TDMI CPU** — full ARM + Thumb instruction set
- **PPU** — all 6 BG modes, OBJ sprites, windows, mosaic, alpha blending
- **Memory** — BIOS, EWRAM, IWRAM, VRAM, OAM, palette, GamePak ROM with mirrors
- **Save** — SRAM, Flash 64K/128K, EEPROM
- **DMA** — 4 channels with immediate/VBlank/HBlank/special timing
- **Timers** — 4 timers with cascade mode
- **Input** — all 10 GBA buttons
- **BIOS** — HLE BIOS with SWI dispatch (Div, CpuSet, LZ77/RL decompression, IntrWait, etc.)

## Quick Start

```bash
# Headless mode — run N frames and save a BMP screenshot
cargo run --release -- "game.gba" --frames 1200 --output screenshot.bmp

# GUI mode — interactive window with keyboard controls
cargo run --release --features gui -- "game.gba" --gui
```

### GUI Controls

| Key | GBA |
|-----|-----|
| Arrow keys | D-Pad |
| Z | A |
| X | B |
| Enter | Start |
| Right Shift | Select |
| A | L |
| S | R |
| R | Reset |
| ESC | Quit |

### Command-Line Options

```
rgba <rom_path> [options]

  --bios <path>       Load BIOS from file
  --frames <N>        Number of frames to run (headless, default: 60)
  --output <path>     BMP output path (default: output.bmp)
  --save-type <type>  Save type: sram, flash64, flash128, eeprom512, eeprom8k
  --gui               Run with graphical window (requires --features gui)
```

## Architecture

```
src/
├── main.rs     — CLI entry point, headless/GUI modes
├── lib.rs      — Gba struct, frame/scanline stepping, PPU compositing
├── cpu.rs      — ARM7TDMI CPU (ARM + Thumb decode/execute, SWI HLE)
├── mem.rs      — Memory bus, IO registers, save types
├── ppu.rs      — Pixel Processing Unit (BG/OBJ rendering)
├── apu.rs      — Audio Processing Unit (stub)
├── dma.rs      — DMA controller (4 channels)
├── timer.rs    — Timer units (4 channels)
├── input.rs    — Keypad input
├── flash.rs    — Flash memory emulation
└── eeprom.rs   — EEPROM emulation
```

## Performance

On a modern machine the emulator runs at approximately **100–110 FPS** in headless
mode and **80 FPS** with per-frame rendering — well above the GBA's native 59.7 FPS.

## Testing

```bash
# Unit + behavior tests
cargo test

# Integration test with a real ROM (requires env var)
RGBA_ROM_PATH="/path/to/game.gba" cargo test -- --ignored --nocapture
```

## License

MIT

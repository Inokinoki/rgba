# Ralph Loop Progress - GBA ROM Testing

## Current Status

The emulator can now execute ARM and Thumb instructions from ROMs, but the gba-tests require proper interrupt/VBlank handling to complete.

## What's Been Fixed

1. **ARM Branch Instruction** - Fixed sign extension and decoding (branches in category 2 weren't being detected)
2. **SBC/RSC Instructions** - Fixed overflow handling
3. **BIOS Function Stubs** - Added SWI handlers for common BIOS functions:
   - SoftReset (0x00)
   - RegisterRamReset (0x01)
   - Halt/Stop (0x02/0x03)
   - IntrWait (0x04)
   - VBlankIntrWait (0x05)
   - Div/DivArm (0x06/0x08)
   - Sqrt (0x0E)

## What's Still Needed

### Critical: VBlank Interrupt Handling

The gba-tests use `VBlankIntrWait` to synchronize frame rendering. Currently this is a no-op stub, so tests run in a tight loop.

Required:
1. Implement proper VBlank timing
2. Implement interrupt controller
3. Return from SWI only after interrupt occurs

### Other Issues Found

1. **SWI 0xBF** - Unknown BIOS function being called, needs investigation
2. **Interrupt Controller** - Not implemented at all
3. **VBlank Timing** - PPU needs to generate VBlank interrupts

## Test Results

### bios.gba
- Executes ARM code successfully
- Calls BIOS functions (SWI 0xBF is unknown but doesn't crash)
- Stalls waiting for VBlank

### arm.gba
- Executes ARM test instructions
- Sets up display (DISPCNT = 0x0080)
- Stalls after 57 instructions waiting for VBlank

### ppu/hello.gba
- Executes and jumps to IWRAM
- VRAM remains empty (likely waiting for VBlank before drawing)

## Next Steps

To make the tests pass, we need:

1. **Interrupt Controller** (src/interrupts.rs):
   - Implement IE, IF, IME registers
   - Handle interrupt requests and acknowledgment

2. **VBlank Timing** (in PPU):
   - Generate VBlank interrupt every frame (scanline 160-227)
   - Set DISPSTAT bit 1 when in VBlank

3. **VBlankIntrWait Implementation**:
   - Wait until VBlank interrupt occurs
   - Return from SWI only after interrupt

4. **Investigate SWI 0xBF**:
   - Unknown BIOS function
   - May be specific to certain ROM types

## Files Modified This Session

- src/cpu.rs: Fixed branch decoding, SBC/RSC, added BIOS SWI handlers
- src/lib.rs: Added memory accessor methods
- examples/trace_execution.rs: Created diagnostic tool
- examples/test_branch.rs: Created branch test tool
- examples/diagnose_display.rs: Created display diagnostic tool
- examples/test_graphics.rs: Created graphics test tool

## How to Run Tests

```bash
# Trace execution
cargo run --example trace_execution -- /path/to/test.gba

# Run in GUI
cargo run --example gui_emulator --features gui -- /path/to/test.gba
```

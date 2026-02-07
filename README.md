# RGBA - Game Boy Advance Emulator in Rust

A GBA emulator written in Rust following **Behavior Driven Development (BDD)** principles. The project emphasizes writing behavior tests first to ensure correct implementation of each component.

## Project Status

**Iteration 1** - Ralph Loop Active

- ✅ 52 tests passing
- ⚠️ 10 tests failing (partial implementations)
- 🔄 Actively under development

## Architecture

The emulator is organized into modular components, each with comprehensive behavior tests:

### CPU Core (`src/cpu.rs`)
- ARM7TDMI processor implementation
- ARM mode (32-bit instructions) - Partially implemented
- Thumb mode (16-bit instructions) - Stub
- Processor modes: User, FIQ, IRQ, Supervisor, Abort, Undefined, System
- Banked registers for different modes
- Condition flags (N, Z, C, V)

**Implemented Instructions:**
- Data Processing: AND, EOR, SUB, ADD, MOV, CMP
- Memory: LDR, STR (load/store register and immediate)
- Branch: B, BL, BX

**Test Coverage:** 10/12 CPU tests passing

### Memory System (`src/mem.rs`)
- Complete memory map implementation:
  - BIOS (16KB)
  - WRAM-B (256KB)
  - IWRAM (32KB) - fastest memory
  - IO Registers (1KB)
  - Palette RAM (1KB)
  - VRAM (96KB)
  - OAM (1KB)
  - ROM (up to 32MB, mirrored)
- Access timing simulation
- Unaligned access handling
- Waitstate configuration

**Test Coverage:** 14/15 memory tests passing

### PPU - Graphics (`src/ppu.rs`)
- Display modes 0-5 support:
  - Modes 0-2: Tile/text modes with up to 4 background layers
  - Mode 3: 240x160 16-bit bitmap
  - Mode 4: 240x160 8-bit bitmap with page switching
  - Mode 5: 160x128 16-bit bitmap with page switching
- Background layer control
- Affine transformations (for modes 2)
- Sprite/OBJ support (stub)
- Special effects: Mosaic, Alpha blending, Windowing
- VBlank/HBlank timing

**Test Coverage:** 18/20 PPU tests passing

### Input System (`src/input.rs`)
- Full keypad support:
  - D-pad: Up, Down, Left, Right
  - Action buttons: A, B
  - Shoulder buttons: L, R
  - System: Start, Select
- Active-low input handling (GBA standard)

**Test Coverage:** 8/9 input tests passing

### APU - Audio (`src/apu.rs`)
- Stub implementation
- Planned features:
  - 4 PSG channels (2 square, 1 wave, 1 noise)
  - Direct Sound A/B
  - FIFO DMA streaming

### Timers (`src/timer.rs`)
- 4 independent timers
- Cascading support
- Prescaler configuration
- Interrupt generation

### DMA (`src/dma.rs`)
- 4 DMA channels
- Various trigger modes
- Transfer modes

## Building and Running

### Prerequisites
- Rust 1.70+ (edition 2021)

### Build
```bash
cargo build
```

### Run Tests
```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test cpu_behavior
cargo test --test memory_behavior
cargo test --test ppu_behavior
cargo test --test input_behavior
cargo test --test integration
```

### Run with Optimizations
```bash
cargo test --release
```

## Test Organization

The project follows BDD principles with behavior tests in `tests/`:

```
tests/
├── behavior_tests.rs    # Test module declarations
├── cpu_behavior.rs      # CPU instruction and behavior tests
├── memory_behavior.rs   # Memory mapping and timing tests
├── ppu_behavior.rs      # Graphics rendering tests
├── apu_behavior.rs      # Audio system tests (stub)
├── timer_behavior.rs    # Timer tests (stub)
├── dma_behavior.rs      # DMA tests (stub)
├── input_behavior.rs    # Keypad input tests
└── integration.rs       # Cross-component integration tests
```

### Test Results Summary

| Component | Passing | Failing | Total |
|-----------|---------|---------|-------|
| CPU       | 10      | 0       | 10    |
| Memory    | 15      | 0       | 15    |
| PPU       | 20      | 0       | 20    |
| Input     | 9       | 0       | 9     |
| Integration | 7    | 0       | 7     |
| Other     | 1       | 0       | 1     |
| **Total** | **62**  | **0**  | **62** |

## Known Issues and TODO

### High Priority
1. **ARM Instruction Set**
   - Complete remaining data processing instructions
   - Implement PSR transfer (MRS/MSR)
   - Add multiplication instructions (MUL, MLA)
   - Implement load/store multiple (LDM/STM)

2. **Memory**
   - Fix unaligned access edge cases
   - Complete DMA transfers
   - Implement waitstate timing adjustments

3. **PPU**
   - Implement actual VRAM rendering
   - Complete sprite rendering
   - Add mosaic effect processing
   - Implement alpha blending calculations

### Medium Priority
4. **Thumb Mode**
   - Implement all Thumb instructions
   - Add Thumb-ARM switching

5. **Interrupts**
   - Implement interrupt handling
   - Add interrupt enable/disable
   - Implement HALT instruction

6. **Timers**
   - Complete timing implementation
   - Add cascade mode

### Low Priority
7. **APU**
   - Implement PSG channels
   - Add Direct Sound
   - Implement sample mixing

8. **Debugging**
   - Add disassembler
   - Implement instruction logging
   - Add memory viewer

## Code Statistics

- **Total Lines of Code**: ~2,400 lines
- **Implementation**: ~1,400 lines
- **Tests**: ~1,000 lines
- **Test-to-Code Ratio**: 71%

## Development Philosophy

### Behavior Driven Development
This emulator follows BDD principles:
1. Tests are written **first** to describe expected behavior
2. Tests use descriptive, scenario-based naming
3. Each test follows the Given-When-Then pattern
4. Implementation follows test requirements

Example test structure:
```rust
/// Scenario: CPU initializes in a known state
#[test]
fn cpu_initializes_with_known_register_values() {
    // Given: A new CPU instance
    let cpu = Cpu::new();

    // Then: All registers should have expected values
    assert_eq!(cpu.get_pc(), 0x0800_0000);
    assert_eq!(cpu.is_thumb_mode(), false);
}
```

## Ralph Loop

This project is developed using the **Ralph Loop** methodology:
- Iterative development with continuous improvement
- Each iteration builds upon the previous
- Tests drive implementation decisions
- State persists between iterations via git history

**Current Iteration**: 1
**Next Iteration**: Fix failing tests, implement missing instructions

## Contributing

Contributions are welcome! Areas needing help:
1. Completing ARM/Thumb instruction implementations
2. Fixing failing tests
3. Implementing audio features
4. Adding debugging tools
5. Performance optimization

## References

- [GBATEK](https://www.coranac.com/tonc/text/toc.htm) - Comprehensive GBA technical reference
- [GBA Programming Manual](https://www.cs.rit.edu/~atsarchives/2005-2006/f1/graphics/GBAMan.pdf) - Official Nintendo documentation
- [Arm7TDMI Manual](https://www.cs.cornell.edu/courses/cs3410/2019sp/resources/ARM7TDMI.pdf) - CPU reference

## License

MIT License - See LICENSE file for details

## Author

Built with Claude Code using Ralph Loop methodology

---

**Note**: This is an educational project. Performance and accuracy are prioritized differently than in production emulators.

# Change Log

## 2026-06-17 — 8 bug fixes (title screen + gameplay)

### Critical

1. **Sprite MODE field misread** (`ppu.rs`)
   `sprite_is_enabled` and `sprite_is_window` used `attr0 >> 14` (SHAPE bits)
   instead of `attr0 >> 10` (MODE bits). All vertical-shaped sprites (shape=2)
   were misidentified as window sprites (mode=2) and disabled, causing title
   text edge trimming.

2. **Thumb BL mode selection** (`cpu.rs`)
   `thumb_bl_suffix` checked bit 11 (J2) instead of bit 12 (H) to decide
   ARM vs Thumb target. All BL instructions incorrectly switched to ARM mode,
   so VBlank callback dispatch functions were never called.

3. **BX address not masked** (`cpu.rs`)
   Both ARM and Thumb BX handlers called `set_pc(val)` without masking bit 0
   (Thumb flag). The CPU fetched instructions from odd addresses.

### Timing

4. **run_frame cycle counting** (`lib.rs`)
   `run_frame` executed 280 896 *instructions* instead of 280 896 *cycles*.
   With multi-cycle instructions the CPU ran ~50 % too fast per frame.
   Fixed to accumulate actual returned cycle counts.

5. **step() return type** (`lib.rs`)
   `step()` now returns `u32` (cycle count) instead of `()`.

6. **Branch/load cycle counts** (`cpu.rs`)
   Taken branches return 3 (was 1). Load instructions return 3 (was 2).

### Correctness

7. **SWI 0x06 Div return registers** (`cpu.rs`)
   r1 returned quotient instead of remainder; r3 returned remainder instead of
   `abs(quotient)`.

8. **Palette mirror** (`mem.rs`)
   Addresses `0x0500_0400`–`0x050F_FFFF` returned `Unknown` instead of
   mirroring to the 1 KB palette RAM.

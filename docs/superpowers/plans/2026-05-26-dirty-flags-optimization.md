# Dirty Flags Optimization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Improve gui_emulator FPS from ~0.5 to ~30 by adding dirty flags to skip redundant per-step synchronization.

**Architecture:** Add dirty flag fields to Memory struct. Set flags on CPU writes via `write_byte_internal()`. Check flags in sync functions, skip if not dirty. Clear flags after sync.

**Tech Stack:** Rust, no new dependencies

---

## Files to Modify

| File | Changes |
|------|---------|
| `src/mem.rs:179-224` | Add dirty flag fields to Memory struct |
| `src/mem.rs:511-543` | Set dirty flags in `write_byte_internal()` |
| `src/lib.rs:431-463` | Gate `sync_ppu()` on dirty flags |
| `src/lib.rs:467-479` | Gate `sync_ppu_to_mem()` on dirty flag |
| `src/lib.rs:482-521` | Gate `sync_io_to_components()` on dirty flags |

---

### Task 1: Add Dirty Flag Fields to Memory

**Files:**
- Modify: `src/mem.rs:179-224`

- [ ] **Step 1: Add dirty flag fields to Memory struct**

In `src/mem.rs`, add after line 218 (`pub halt_pending: bool`):

```rust
    // Dirty flags for lazy synchronization
    pub vram_dirty: bool,
    pub oam_dirty: bool,
    pub palette_dirty: bool,
    pub io_ppu_dirty: bool,
    pub io_timer_dirty: bool,
    pub io_dma_dirty: bool,
```

- [ ] **Step 2: Initialize dirty flags in Memory::new()**

Find the `Memory::new()` constructor (around line 230) and add initialization:

```rust
            vram_dirty: true,
            oam_dirty: true,
            palette_dirty: true,
            io_ppu_dirty: true,
            io_timer_dirty: true,
            io_dma_dirty: true,
```

Use `true` initially so first sync always runs.

- [ ] **Step 3: Verify compilation**

Run: `cargo build 2>&1 | grep "^error"`
Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add src/mem.rs
git commit -m "feat: add dirty flag fields to Memory struct"
```

---

### Task 2: Set Dirty Flags on Memory Writes

**Files:**
- Modify: `src/mem.rs:511-543` (`write_byte_internal`)

- [ ] **Step 1: Add dirty flag logic to write_byte_internal**

In `write_byte_internal()`, after the match block that writes to each region, add dirty flag setting. The match is around line 512-542. Add before the closing `}` of the function:

```rust
        // Set dirty flags based on written address
        match addr {
            0x06000000..=0x06017FFF => self.vram_dirty = true,
            0x07000000..=0x070003FF => self.oam_dirty = true,
            0x05000000..=0x050003FF => self.palette_dirty = true,
            0x04000000..=0x04000055 => self.io_ppu_dirty = true,
            0x04000100..=0x0400010F => self.io_timer_dirty = true,
            0x040000B0..=0x040000DF => self.io_dma_dirty = true,
            _ => {}
        }
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build 2>&1 | grep "^error"`
Expected: no errors

- [ ] **Step 3: Run ROM tests to verify correctness**

Run: `cargo run --example run_all_gba_tests 2>&1 | tail -5`
Expected: `13 passed, 0 failed`

- [ ] **Step 4: Commit**

```bash
git add src/mem.rs
git commit -m "feat: set dirty flags on memory writes"
```

---

### Task 3: Gate sync_ppu() on Dirty Flags

**Files:**
- Modify: `src/lib.rs:431-463` (`sync_ppu`)

- [ ] **Step 1: Wrap sync_ppu with dirty checks**

Replace the entire `sync_ppu()` method with:

```rust
    pub fn sync_ppu(&mut self) {
        let has_io = self.mem.io_ppu_dirty;
        let has_vram = self.mem.vram_dirty;
        let has_oam = self.mem.oam_dirty;

        if !has_io && !has_vram && !has_oam {
            return; // Nothing to sync
        }

        if has_vram {
            self.ppu.sync_vram(self.mem.vram());
            self.mem.vram_dirty = false;
        }

        if has_oam {
            self.ppu.sync_oam(self.mem.oam());
            self.mem.oam_dirty = false;
        }

        if has_io {
            let io = self.mem.io();
            self.ppu.set_dispcnt(u16::from_le_bytes([io[0], io[1]]));

            for bg in 0..4 {
                let off = 8 + bg * 2;
                self.ppu.set_bgcnt(bg, u16::from_le_bytes([io[off], io[off + 1]]));
            }

            for bg in 0..4 {
                let h_off = 16 + bg * 4;
                let v_off = h_off + 2;
                self.ppu.set_bg_hofs(bg, u16::from_le_bytes([io[h_off], io[h_off + 1]]) & 0x1FF);
                self.ppu.set_bg_vofs(bg, u16::from_le_bytes([io[v_off], io[v_off + 1]]) & 0x1FF);
            }

            self.ppu.set_blend_control(u16::from_le_bytes([io[0x50], io[0x51]]));
            self.ppu.set_blend_alpha(u16::from_le_bytes([io[0x52], io[0x53]]));
            self.ppu.set_blend_brightness(u16::from_le_bytes([io[0x54], io[0x55]]));

            self.ppu.bg_mosaic = u16::from_le_bytes([io[0x4C], io[0x4D]]);
            self.ppu.obj_mosaic = u16::from_le_bytes([io[0x4E], io[0x4F]]);

            self.mem.io_ppu_dirty = false;
        }
    }
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build 2>&1 | grep "^error"`
Expected: no errors

- [ ] **Step 3: Run ROM tests**

Run: `cargo run --example run_all_gba_tests 2>&1 | tail -5`
Expected: `13 passed, 0 failed`

- [ ] **Step 4: Commit**

```bash
git add src/lib.rs
git commit -m "feat: gate sync_ppu on dirty flags"
```

---

### Task 4: Gate sync_io_to_components() on Dirty Flags

**Files:**
- Modify: `src/lib.rs:482-521` (`sync_io_to_components`)

- [ ] **Step 1: Wrap sync_io_to_components with dirty checks**

Replace the method with:

```rust
    fn sync_io_to_components(&mut self) {
        if self.mem.io_timer_dirty {
            let io = self.mem.io();
            for i in 0..4 {
                let base = 0x100 + (i * 4);
                let control = u16::from_le_bytes([io[base + 2], io[base + 3]]);
                let reload = u16::from_le_bytes([io[base], io[base + 1]]);
                self.timers[i].set_control(control);
                self.timers[i].set_reload(reload);
            }
            self.mem.io_timer_dirty = false;
        }

        if self.mem.io_dma_dirty {
            let io = self.mem.io();
            for i in 0..4 {
                let base = 0xB0 + (i * 12);
                let src = u32::from_le_bytes([io[base], io[base + 1], io[base + 2], io[base + 3]]);
                let dst = u32::from_le_bytes([io[base + 4], io[base + 5], io[base + 6], io[base + 7]]);
                let count = u16::from_le_bytes([io[base + 8], io[base + 9]]);
                let control = u16::from_le_bytes([io[base + 10], io[base + 11]]);
                self.dma[i].set_src_addr(src);
                self.dma[i].set_dst_addr(dst);
                self.dma[i].set_count(count);
                self.dma[i].set_control(control);
            }
            self.mem.io_dma_dirty = false;
        }
    }
```

Note: Blend registers were duplicated in both `sync_io_to_components` and `sync_ppu`. Keep them only in `sync_ppu` (already gated by `io_ppu_dirty`).

- [ ] **Step 2: Verify compilation**

Run: `cargo build 2>&1 | grep "^error"`
Expected: no errors

- [ ] **Step 3: Run ROM tests**

Run: `cargo run --example run_all_gba_tests 2>&1 | tail -5`
Expected: `13 passed, 0 failed`

- [ ] **Step 4: Commit**

```bash
git add src/lib.rs
git commit -m "feat: gate sync_io_to_components on dirty flags"
```

---

### Task 5: Optimize sync_ppu_to_mem() Calls

**Files:**
- Modify: `src/lib.rs:217-317` (`step`)

- [ ] **Step 1: Remove redundant sync_ppu_to_mem call**

In `step()`, the second `sync_ppu_to_mem()` call at line 268 is only needed after PPU stepping. Keep it but gate on whether PPU actually stepped:

```rust
        // Step PPU and check for VBlank/HBlank interrupts
        let (vblank_start, hblank_start) = self.ppu.step_vblank_check(cycles);
        if vblank_start {
            self.mem.interrupt.request(Interrupt::VBLANK);
        }
        if hblank_start {
            self.mem.interrupt.request(Interrupt::HBLANK);
        }

        // Only sync PPU state back if PPU actually stepped (cycles > 0)
        if cycles > 0 {
            self.sync_ppu_to_mem();
        }
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build 2>&1 | grep "^error"`
Expected: no errors

- [ ] **Step 3: Run ROM tests**

Run: `cargo run --example run_all_gba_tests 2>&1 | tail -5`
Expected: `13 passed, 0 failed`

- [ ] **Step 4: Commit**

```bash
git add src/lib.rs
git commit -m "opt: skip sync_ppu_to_mem when CPU is halted"
```

---

### Task 6: Benchmark and Verify

- [ ] **Step 1: Build release and run gui_emulator**

Run: `time cargo run --release --example gui_emulator --features gui -- --frames 60 gba-tests/ppu/hello.gba 2>&1 | tail -10`
Expected: FPS significantly higher than 0.5, verification PASS

- [ ] **Step 2: Run full ROM test suite**

Run: `cargo run --release --example run_all_gba_tests 2>&1 | tail -5`
Expected: `13 passed, 0 failed`

- [ ] **Step 3: Run CI examples build**

Run: `cargo build --examples 2>&1 | grep "^error"`
Expected: no errors

- [ ] **Step 4: Commit final state**

```bash
git add -A
git commit -m "feat: dirty flags optimization complete - target 30 FPS"
```

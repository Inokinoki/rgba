# Dirty Flags Optimization Design

## Goal

Improve gui_emulator FPS from ~0.5 to ~30 by eliminating redundant per-step synchronization.

## Problem

Current `gba.step()` performs 3 full syncs every single step (280k steps/frame):

1. `sync_ppu()` — copies entire VRAM (96KB) + OAM (1KB) + all BG registers
2. `sync_ppu_to_mem()` — called TWICE per step
3. `sync_io_to_components()` — syncs all Timer + DMA registers

Most steps don't modify these regions, so the syncs are wasted work.

## Solution: Dirty Flags

Add dirty flags to `Memory` struct. Set flags on CPU writes, check flags before syncs.

### Dirty Regions

| Region | Address Range | Affected Sync |
|--------|---------------|---------------|
| VRAM | 0x06000000-0x06017FFF | `sync_vram()` |
| OAM | 0x07000000-0x070003FF | `sync_oam()` |
| IO (PPU) | 0x04000000-0x04000055 | `sync_ppu()` BG/blend registers |
| IO (Timer) | 0x04000100-0x0400010F | `sync_io_to_components()` timers |
| IO (DMA) | 0x040000B0-0x040000DF | `sync_io_to_components()` DMA |

### Changes

#### 1. Memory struct (`src/mem.rs`)

Add fields:
```rust
pub struct Memory {
    // ... existing fields ...
    pub vram_dirty: bool,
    pub oam_dirty: bool,
    pub io_ppu_dirty: bool,
    pub io_timer_dirty: bool,
    pub io_dma_dirty: bool,
}
```

Add method:
```rust
pub fn mark_dirty(&mut self, addr: u32) {
    match addr {
        0x06000000..=0x06017FFF => self.vram_dirty = true,
        0x07000000..=0x070003FF => self.oam_dirty = true,
        0x04000000..=0x04000055 => self.io_ppu_dirty = true,
        0x04000100..=0x0400010F => self.io_timer_dirty = true,
        0x040000B0..=0x040000DF => self.io_dma_dirty = true,
        _ => {}
    }
}
```

#### 2. CPU writes (`src/cpu.rs`)

All memory write paths must call `mem.mark_dirty(addr)` after writing. Key locations:
- `write_byte()`, `write_half()`, `write_word()` in Memory
- DMA transfer writes

#### 3. Sync functions (`src/lib.rs`)

```rust
fn sync_ppu(&mut self) {
    if self.mem.vram_dirty {
        self.ppu.sync_vram(self.mem.vram());
        self.mem.vram_dirty = false;
    }
    if self.mem.oam_dirty {
        self.ppu.sync_oam(self.mem.oam());
        self.mem.oam_dirty = false;
    }
    if self.mem.io_ppu_dirty {
        // sync BG registers, blend, etc.
        self.mem.io_ppu_dirty = false;
    }
}

fn sync_io_to_components(&mut self) {
    if self.mem.io_timer_dirty {
        // sync timers
        self.mem.io_timer_dirty = false;
    }
    if self.mem.io_dma_dirty {
        // sync DMA
        self.mem.io_dma_dirty = false;
    }
}
```

### Expected Performance

- Best case (no writes): 5 bool checks per step → ~0 overhead
- Worst case (writes every step): same as current
- Typical ROM: writes ~1% of steps → ~100x speedup on syncs
- Target: 0.5 FPS → 30+ FPS

### Testing

All 13 ROM tests must still pass. gui_emulator must render correct画面 at higher FPS.

# Ralph Loop Progress - GBA ROM Testing

## Current Status

**Investigating ARM execution bugs.** The emulator has issues with ARM instruction execution that cause invalid PC jumps.

**Test Results:**
- ❌ **arm.gba**: Jumps to invalid address outside ROM (0x0807A7B0)
- ❌ **thumb.gba**: Invalid PC (0xD23F2FA9) - Thumb BLX issue
- ❌ **bios.gba**: Stalls at PC 0x08000264 due to unknown SWI 0xBF
- ❌ **memory.gba**: Stalls early
- ❌ **unsafe.gba**: Invalid PC (0x0ACD9F08)

## Key Findings

### ARM Mode Execution Issue (arm.gba)
The CPU starts in ARM mode correctly but eventually jumps to an invalid address:
- Step 28: PC jumps from 0x08001FA4 to 0x08001972 (non-word-aligned!)
- Step 29-31: PC remains non-word-aligned (0x08001972, 0x08001986, 0x0800198E)
- Step 31: Large jump to 0x0807A7B0 (outside ROM bounds)

**Root Cause**: The PC becomes non-word-aligned in ARM mode, which is incorrect. ARM instructions should only execute at word-aligned addresses. This suggests:
1. An ARM instruction is branching to a non-word-aligned address
2. OR there's a bug in how PC is being updated

### Thumb BLX Instruction
The Thumb BL/BLX instruction implementation was partially fixed:
- Fixed Category 7 dispatch to check top 5 bits for proper format
- Rewrote BL/BLX offset calculation
- Still has bugs that cause incorrect target calculation

## Recent Changes

### CPU (src/cpu.rs)
- Fixed Category 7 instruction dispatch to properly validate BL/BLX format
- Rewrote `thumb_bl_prefix()` with proper sign bit handling
- Rewrote `thumb_bl_suffix()` with correct offset calculation
- Added undefined instruction handling for Category 7

### Known Bugs

1. **ARM PC Alignment**: PC becomes non-word-aligned in ARM mode
2. **Thumb BLX Target**: Incorrect target calculation
3. **SWI 0xBF**: Unknown BIOS call (possibly test-specific)

## Next Steps

To fix the remaining issues:

1. **Trace ARM execution** - Find which instruction causes the non-word-aligned PC
2. **Fix BL/BLX calculation** - Verify offset formula with ARM ARM
3. **Check instruction fetch** - Verify pipeline is working correctly
4. **Handle SWI 0xBF** - Investigate if this needs special handling

## Test Execution

```bash
# Run all tests
cargo run --quiet --example test_all_roms 2>/dev/null

# Trace individual test
cargo run --quiet --example trace_execution -- /tmp/gba-tests/arm/arm.gba 2>/dev/null
```

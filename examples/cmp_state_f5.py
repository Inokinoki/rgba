#!/usr/bin/env python3
"""Compare EWRAM + IWRAM between our emulator and mGBA at frame 5."""
import subprocess, struct, sys, os

ROM = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba"
RGBA_DIR = "/home/ubuntu/Builds/RGBA"

def run_rgba_dump():
    """Run a quick Rust example to dump EWRAM + IWRAM at frame 5."""
    code = r'''
use rgba::Gba;
use std::io::Write;
fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("%s").unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];
    for _ in 0..5 { gba.run_frame_parallel(&mut framebuffer); }
    
    let wram = gba.mem().wram();
    let iwram = gba.mem().iwram();
    
    let mut f = std::fs::File::create("/tmp/rgba_state_f5.bin").unwrap();
    // Write EWRAM (256KB)
    f.write_all(&wram[..]).unwrap();
    // Write IWRAM (32KB)
    f.write_all(&iwram[..]).unwrap();
    
    // Also dump registers
    let io = gba.mem().io();
    f.write_all(&io[..]).unwrap();
    
    let ie = gba.mem().interrupt.read_register(0x200);
    let irf = gba.mem().interrupt.read_register(0x002);
    let ime = gba.mem().interrupt.read_register(0x208);
    f.write_all(&ie.to_le_bytes()).unwrap();
    f.write_all(&irf.to_le_bytes()).unwrap();
    f.write_all(&ime.to_le_bytes()).unwrap();
    
    eprintln!("Dumped state: EWRAM={} IWRAM={} IO={}", wram.len(), iwram.len(), io.len());
}
''' % ROM
    
    with open(f"{RGBA_DIR}/examples/_dump_state.rs", "w") as f:
        f.write(code)
    
    result = subprocess.run(
        ["cargo", "run", "--release", "--example", "_dump_state"],
        cwd=RGBA_DIR, capture_output=True, text=True, timeout=120
    )
    if result.returncode != 0:
        print("Build error:", result.stderr[-500:])
        sys.exit(1)
    
    return "/tmp/rgba_state_f5.bin"

def run_mgba_dump():
    """Use mGBA debugger to dump EWRAM + IWRAM at frame 5."""
    # Use the mGBA script approach: run for 5 frames, dump memory
    script = f"""
b 0x080D2F10
c
c
c
c
c
c
x/262144b 0x02000000
x/32768b 0x03000000
q
"""
    
    with open("/tmp/mgba_dump_script.txt", "w") as f:
        f.write(script)
    
    # Run mGBA with debugger
    env = os.environ.copy()
    env["DISPLAY"] = ":99"
    
    proc = subprocess.run(
        ["/usr/games/mgba", "-d", ROM],
        input=script, capture_output=True, text=True, timeout=30, env=env
    )
    
    # Parse the output to extract memory dumps
    # This is tricky because mGBA's output format varies
    # Let's just save the raw output for manual inspection
    with open("/tmp/mgba_dump_output.txt", "w") as f:
        f.write(proc.stdout)
    
    return proc.stdout

# First dump our state
print("=== Dumping RGBA state at frame 5 ===")
rgba_file = run_rgba_dump()

# Read our state
with open(rgba_file, "rb") as f:
    rgba_data = f.read()
    
rgba_ewram = rgba_data[:0x40000]
rgba_iwram = rgba_data[0x40000:0x48000]

print(f"RGBA EWRAM: {len(rgba_ewram)} bytes")
print(f"RGBA IWRAM: {len(rgba_iwram)} bytes")

# Print key state values
print("\n=== Key EWRAM values (RGBA) ===")
# IO buffer at 0x02008D2C
buf_off = 0x8D2C
for i in range(4):
    off = buf_off + i * 4
    val = struct.unpack_from('<I', rgba_ewram, off)[0]
    print(f"  0x{0x02008D2C + i*4:08X}: 0x{val:08X}")

print("\n=== Key IWRAM values (RGBA) ===")
for addr in [0x0410, 0x0430, 0x0450, 0x0958, 0x7FF8, 0x7FFC]:
    val = struct.unpack_from('<I', rgba_iwram, addr)[0]
    print(f"  0x{0x03000000+addr:08X}: 0x{val:08X}")

# Compare EWRAM - check for large non-zero regions
print("\n=== Non-zero EWRAM regions (RGBA) ===")
in_region = False
start = 0
for i in range(0, len(rgba_ewram), 4):
    val = struct.unpack_from('<I', rgba_ewram, i)[0]
    if val != 0 and not in_region:
        start = i
        in_region = True
    elif val == 0 and in_region:
        if i - start >= 16:  # Only report regions >= 16 bytes
            print(f"  0x{0x02000000+start:08X}-0x{0x02000000+i:08X} ({i-start} bytes)")
        in_region = False
if in_region and len(rgba_ewram) - start >= 16:
    print(f"  0x{0x02000000+start:08X}-0x{0x02000000+len(rgba_ewram):08X} ({len(rgba_ewram)-start} bytes)")

# Check specific game structure addresses
print("\n=== Game structure at EWRAM 0x0200XXXX ===")
for base in [0x9200, 0x9208, 0x9210, 0x9218, 0x9220, 0x9228]:
    for i in range(8):
        off = base + i * 4
        val = struct.unpack_from('<I', rgba_ewram, off)[0]
        if val != 0:
            print(f"  0x{0x02000000+off:08X}: 0x{val:08X}")

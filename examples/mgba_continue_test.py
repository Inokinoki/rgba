#!/usr/bin/env python3
"""
Use mGBA debugger with continue + timed break to advance game.
"""

import pty, os, select, time, subprocess, signal

ROM = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba"
IWSTART = 0x03006DD8


def ru(fd, marker, timeout=30):
    buf = b""
    t0 = time.time()
    while time.time() - t0 < timeout:
        r, _, _ = select.select([fd], [], [], 0.05)
        if r:
            try:
                d = os.read(fd, 4096)
                if not d:
                    break
                buf += d
                if marker in buf:
                    return buf
            except:
                break
    return buf


def sc(fd, cmd):
    os.write(fd, (cmd + "\n").encode())


def read_hw(fd, addr):
    sc(fd, f"r/2 0x{addr:08X}")
    buf = ru(fd, b">", timeout=3)
    text = buf.decode(errors="replace")
    for line in text.split("\n"):
        line = line.strip()
        if "=" in line:
            parts = line.split("=")
            if len(parts) >= 2:
                try:
                    return int(parts[-1].strip(), 16)
                except:
                    pass
    return 0xFFFF


mf, sf = pty.openpty()
env = os.environ.copy()
env["SDL_VIDEODRIVER"] = "dummy"
env["SDL_AUDIODRIVER"] = "dummy"
proc = subprocess.Popen(
    ["/usr/games/mgba", "-d", ROM], stdin=sf, stdout=sf, stderr=sf, env=env
)
os.close(sf)

print("Waiting for debugger...")
buf = ru(mf, b">", timeout=15)
print("Ready.")
print("Initial output:", buf.decode(errors="replace")[:300])

# Try using continue + break approach
# Set a breakpoint at VBlank (0x04000004 DISPSTAT bit 3 = VBlank interrupt)
# Or better, just continue for a few seconds and check

# First, check if we're at the right spot
sc(mf, "regs")
buf = ru(mf, b">", timeout=5)
print("\nRegisters:")
print(buf.decode(errors="replace"))

# Check PC
sc(mf, "r/4 0x03007FFC")
buf = ru(mf, b">", timeout=3)
print("IRQ handler ptr:", buf.decode(errors="replace"))

# Try continue for real time, then Ctrl+C to break
sc(mf, "c")
time.sleep(5)  # Let game run for 5 seconds (~300 frames)
# Send Ctrl+C to break
os.write(mf, b"\x03")
buf = ru(mf, b">", timeout=5)
print("\nAfter 5s continue:")
print(buf.decode(errors="replace")[:500])

# Check state now
dispcnt = read_hw(mf, 0x04000000)
print(f"\nDISPCNT: 0x{dispcnt:04X}")

# Read IWRAM entries
print("\nFirst 32 IWRAM entries at 0x03006DD8:")
for i in range(32):
    val = read_hw(mf, IWSTART + i * 2)
    if i % 16 == 0:
        print(f"\n  [{i:03X}]: ", end="")
    print(f"{val:04X} ", end="")
print()

# Also check a broader IWRAM region for any non-FF
print("\nIWRAM non-FF scan (coarse, 64-byte blocks):")
for block in range(0, 0x8000, 64):
    non_ff = 0
    for off in range(0, 64, 2):
        val = read_hw(mf, 0x03000000 + block + off)
        if val != 0xFFFF:
            non_ff += 1
    if non_ff > 0:
        # Print first few values
        vals = []
        for off in range(0, min(16, 64), 2):
            vals.append(f"{read_hw(mf, 0x03000000 + block + off):04X}")
        print(f"  0x{0x03000000 + block:08X}: {non_ff}/32 non-FF  [{', '.join(vals)}]")

sc(mf, "quit")
time.sleep(0.5)
proc.kill()
os.close(mf)

#!/usr/bin/env python3
"""Use mGBA debugger to read VRAM tiles"""

import subprocess
import time
import select
import os
import pty
import re
import struct

ROM = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba"
env = os.environ.copy()
env["SDL_VIDEODRIVER"] = "dummy"
env["SDL_AUDIODRIVER"] = "dummy"
env["PATH"] = "/usr/games:" + env.get("PATH", "")

master_fd, slave_fd = pty.openpty()
proc = subprocess.Popen(
    ["mgba", "-d", ROM],
    stdin=subprocess.PIPE,
    stdout=slave_fd,
    stderr=slave_fd,
    env=env,
)
os.close(slave_fd)


def read_pty(timeout=0.5):
    buf = b""
    while select.select([master_fd], [], [], timeout)[0]:
        try:
            chunk = os.read(master_fd, 65536)
            if not chunk:
                break
            buf += chunk
        except:
            break
    return buf.decode(errors="replace")


time.sleep(1)
read_pty(0.5)

# Advance 240 frames using 'trace' command (fast bulk execution)
# Or use 'c' (continue) with a breakpoint at VBlank
# Actually, let me just run frame by frame using continue
# First set a breakpoint after VBlank
# Or simpler: advance by using the trace command with a large count

# Actually, mGBA doesn't have a 'frame' command. Let me use 'trace' to advance.
# trace N = execute N instructions
# At ~16.78 MHz, one frame = ~280896 cycles, but in THUMB ~1 instruction/cycle
# So ~280K instructions per frame, 240 frames = ~67M instructions

# That's too many for trace. Let me try setting a VBlank breakpoint and continuing
# VBlank IRQ handler address... or we can just use continue and break after 240 VBlanks

# Actually, let me just use the 'c' (continue) command and let it run for a while
# then break with Ctrl+C equivalent

# Set breakpoint at VBlank handler
# Read the IRQ handler address from 0x03007FFC
proc.stdin.write(b"r/4 0x03007FFC\n")
proc.stdin.flush()
time.sleep(0.3)
out = read_pty(0.3)
print(f"IRQ handler read: {out.strip()}")

# Try the 'c' command to continue, then wait a bit
proc.stdin.write(b"c\n")
proc.stdin.flush()
time.sleep(5)  # Let it run for ~5 seconds = ~300 frames at 60fps
read_pty(0.5)  # Drain any output

# Now break into debugger - send Ctrl+C
# mGBA uses '!' to break, or we can set a breakpoint first
# Let's try sending a signal
import signal

proc.send_signal(signal.SIGINT)
time.sleep(0.5)
out = read_pty(0.5)
print(f"After break: {out[:200]}")

# Read VRAM tiles 344-400 (subset for comparison)
print("\n=== Reading VRAM from mGBA ===")
vram_data = {}

for tid in list(range(344, 400)) + list(range(0, 20)):
    addr = 0x06000000 + tid * 32
    # Read 8 words (32 bytes) per tile
    for i in range(8):
        word_addr = addr + i * 4
        proc.stdin.write(f"r/4 0x{word_addr:08X}\n".encode())
        proc.stdin.flush()
        time.sleep(0.01)

    # Collect output
    out = read_pty(0.05)
    for line in out.strip().split("\n"):
        m = re.match(r"\s*([0-9A-Fa-f]{8})\s*=\s*0x([0-9A-Fa-f]{8})", line)
        if not m:
            m = re.match(r"\s*=\s*0x([0-9A-Fa-f]{8})", line)
            if m:
                vram_data[word_addr] = int(m.group(1), 16)
        if m and len(m.groups()) == 2:
            a = int(m.group(1), 16)
            v = int(m.group(2), 16)
            vram_data[a] = v

print(f"Read {len(vram_data)} VRAM words")

# Check tiles 344-400
for tid in range(344, 400):
    addr = 0x06000000 + tid * 32
    words = [vram_data.get(addr + i * 4, 0) for i in range(8)]
    has_data = any(w != 0 for w in words)
    if has_data:
        print(f"  mGBA tile {tid}: HAS DATA: {' '.join(f'{w:08X}' for w in words[:4])}")

# Check tiles 0-19
for tid in range(0, 20):
    addr = 0x06000000 + tid * 32
    words = [vram_data.get(addr + i * 4, 0) for i in range(8)]
    has_data = any(w != 0 for w in words)
    if has_data:
        print(f"  mGBA tile {tid}: HAS DATA: {' '.join(f'{w:08X}' for w in words[:4])}")

proc.stdin.write(b"quit\n")
proc.stdin.flush()
try:
    proc.wait(timeout=5)
except:
    proc.kill()
os.close(master_fd)

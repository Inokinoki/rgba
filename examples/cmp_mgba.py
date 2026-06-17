#!/usr/bin/env python3
"""Capture VRAM from mGBA debugger for comparison"""

import subprocess
import time
import re
import struct
import sys

ROM = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba"


def send_cmd(proc, cmd, timeout=5):
    proc.stdin.write((cmd + "\n").encode())
    proc.stdin.flush()
    time.sleep(0.1)


def read_vram_block(proc, start, size):
    """Read a block of VRAM via mGBA debugger"""
    data = bytearray()
    addr = start
    while addr < start + size:
        chunk = min(256, start + size - addr)
        # Use 'x' to examine memory
        proc.stdin.write(f"x/1x 0x{addr:08X}\n".encode())
        proc.stdin.flush()
        time.sleep(0.01)
        addr += 4
    return data


# Launch mGBA in debugger mode
proc = subprocess.Popen(
    ["mgba", "-d", ROM],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.STDOUT,
    env={
        "SDL_VIDEODRIVER": "dummy",
        "SDL_AUDIODRIVER": "dummy",
        "PATH": "/usr/games:" + subprocess.os.environ.get("PATH", ""),
    },
)

time.sleep(1)

# Advance 240 frames
# mGBA debugger uses 'frame' or 'n' commands
for i in range(240):
    proc.stdin.write(b"frame\n")
    proc.stdin.flush()

time.sleep(2)

# Read VRAM at tiles 344-472 (offset 0x2C00-0x3B20)
# Read in 4-byte chunks
# Actually mGBA debugger supports 'xx' for hex dump
# Let me try the 'x' command

# First, let me try a simple approach: dump VRAM using the memory read command
output_lines = []
for offset in range(0, 0x10000, 4):
    addr = 0x06000000 + offset
    cmd = f"x/1w 0x{addr:08X}\n"
    proc.stdin.write(cmd.encode())
    proc.stdin.flush()

time.sleep(5)

# Read all output
proc.stdout.flush()
# Kill mGBA
proc.stdin.write(b"quit\n")
proc.stdin.flush()

try:
    output, _ = proc.communicate(timeout=10)
    # Parse VRAM data from output
    vram = bytearray(0x10000)
    for line in output.decode(errors="replace").split("\n"):
        # Look for lines like: 06000000: 12345678
        m = re.match(r"\s*([0-9A-Fa-f]{8}):\s+([0-9A-Fa-f]{8})", line)
        if m:
            addr = int(m.group(1), 16)
            val = int(m.group(2), 16)
            offset = addr - 0x06000000
            if 0 <= offset < 0x10000:
                struct.pack_into("<I", vram, offset, val)

    # Save VRAM dump
    with open("/tmp/mgba_vram.bin", "wb") as f:
        f.write(vram)

    # Check tiles 344-472
    nonzero = 0
    for tid in range(344, 473):
        off = tid * 32
        if any(vram[off + i] != 0 for i in range(32)):
            nonzero += 1
    print(f"mGBA tiles 344-472 with data: {nonzero}/129")

    # Check tiles 0-343
    nonzero = 0
    for tid in range(0, 344):
        off = tid * 32
        if any(vram[off + i] != 0 for i in range(32)):
            nonzero += 1
    print(f"mGBA tiles 0-343 with data: {nonzero}/344")

except subprocess.TimeoutExpired:
    proc.kill()
    print("Timeout!")

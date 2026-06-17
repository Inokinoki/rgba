#!/usr/bin/env python3
"""Use mGBA debugger to read VRAM tiles 344-472"""

import subprocess
import time
import select
import os
import re

ROM = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba"
env = os.environ.copy()
env["SDL_VIDEODRIVER"] = "dummy"
env["SDL_AUDIODRIVER"] = "dummy"
env["PATH"] = "/usr/games:" + env.get("PATH", "")

proc = subprocess.Popen(
    ["mgba", "-d", ROM],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.STDOUT,
    env=env,
)


def drain():
    buf = b""
    while select.select([proc.stdout], [], [], 0.2)[0]:
        chunk = proc.stdout.read1(65536)
        if not chunk:
            break
        buf += chunk
    return buf


time.sleep(1)
drain()

# Set log level to suppress frame output
proc.stdin.write(b"log 0\n")
proc.stdin.flush()
time.sleep(0.2)
drain()

# Advance 240 frames quietly
for i in range(240):
    proc.stdin.write(b"frame\n")
    proc.stdin.flush()
time.sleep(3)
drain()

# Now read VRAM using x command
# mGBA debugger: x/Nw ADDR reads N words starting at ADDR
# Read tiles 344-472 area (offset 0x2C00-0x3B20)
# That's 129 tiles * 32 bytes = 4128 bytes = 1032 words

# Read in chunks of 16 words
vram_data = {}
for base_off in range(0x2C00, 0x3B20, 64):
    addr = 0x06000000 + base_off
    proc.stdin.write(f"x/16w 0x{addr:08X}\n".encode())
    proc.stdin.flush()
    time.sleep(0.05)

    output = drain().decode(errors="replace")
    for line in output.split("\n"):
        m = re.match(r"\s*([0-9A-Fa-f]{8}):\s+((?:[0-9A-Fa-f]{8}\s*)+)", line)
        if m:
            word_addr = int(m.group(1), 16)
            words = m.group(2).split()
            for j, w in enumerate(words):
                vram_data[word_addr + j * 4] = int(w, 16)

# Also read tiles 0-10 for comparison
for base_off in range(0, 11 * 32, 64):
    addr = 0x06000000 + base_off
    proc.stdin.write(f"x/16w 0x{addr:08X}\n".encode())
    proc.stdin.flush()
    time.sleep(0.05)

    output = drain().decode(errors="replace")
    for line in output.split("\n"):
        m = re.match(r"\s*([0-9A-Fa-f]{8}):\s+((?:[0-9A-Fa-f]{8}\s*)+)", line)
        if m:
            word_addr = int(m.group(1), 16)
            words = m.group(2).split()
            for j, w in enumerate(words):
                vram_data[word_addr + j * 4] = int(w, 16)

proc.stdin.write(b"quit\n")
proc.stdin.flush()
try:
    proc.wait(timeout=5)
except:
    proc.kill()

print(f"Read {len(vram_data)} VRAM words from mGBA")

# Check tiles 344-472
nonzero_tiles = 0
for tid in range(344, 473):
    off = 0x06000000 + tid * 32
    has_data = False
    for i in range(8):
        word_off = off + i * 4
        if word_off in vram_data and vram_data[word_off] != 0:
            has_data = True
            break
    if has_data:
        nonzero_tiles += 1
        print(
            f"  mGBA tile {tid}: HAS DATA (first word: 0x{vram_data.get(off, 0):08X})"
        )

print(f"mGBA tiles 344-472 with data: {nonzero_tiles}/129")

# Check tiles 0-10
print("\nmGBA tiles 0-10:")
for tid in range(11):
    off = 0x06000000 + tid * 32
    first_word = vram_data.get(off, 0)
    print(f"  tile {tid}: first word = 0x{first_word:08X}")

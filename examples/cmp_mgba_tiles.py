#!/usr/bin/env python3
"""Check tile data in mGBA - are tiles 344-472 really empty?"""

import subprocess
import time
import select
import os
import pty
import re

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


def read_pty(timeout=0.3):
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


def read_word(addr):
    proc.stdin.write(f"r/4 0x{addr:08X}\n".encode())
    proc.stdin.flush()
    time.sleep(0.05)
    out = read_pty(0.05)
    m = re.search(r"0x([0-9A-Fa-f]{8})", out)
    if m:
        return int(m.group(1), 16)
    return None


time.sleep(1)
read_pty(0.5)

# Run for ~8 seconds (480 frames)
proc.stdin.write(b"c\n")
proc.stdin.flush()
time.sleep(8)
read_pty(1)

import signal

proc.send_signal(signal.SIGINT)
time.sleep(0.5)
read_pty(0.5)

# Check tile ranges
print("=== mGBA tile data check ===")

# Scan all 1024 tiles
nonzero_tiles = []
for tid in range(1024):
    addr = 0x06000000 + tid * 32
    has_data = False
    for i in range(8):
        w = read_word(addr + i * 4)
        if w is not None and w != 0:
            has_data = True
            break
    if has_data:
        nonzero_tiles.append(tid)

print(f"Total non-zero tiles: {len(nonzero_tiles)} out of 1024")

# Show ranges
if nonzero_tiles:
    ranges = []
    start = nonzero_tiles[0]
    prev = start
    for t in nonzero_tiles[1:]:
        if t != prev + 1:
            ranges.append((start, prev))
            start = t
        prev = t
    ranges.append((start, prev))
    print("Non-zero tile ranges:")
    for s, e in ranges:
        print(f"  Tiles {s}-{e} ({e - s + 1} tiles)")

# Check BG0CNT
bg0cnt = read_word(0x04000008)
print(f"BG0CNT = 0x{bg0cnt:08X}" if bg0cnt else "BG0CNT read failed")

proc.stdin.write(b"quit\n")
proc.stdin.flush()
try:
    proc.wait(timeout=5)
except:
    proc.kill()
os.close(master_fd)

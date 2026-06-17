#!/usr/bin/env python3
"""Read mGBA state at exact frame count"""

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


def read_pty(timeout=0.2):
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


def read_halfword(addr):
    proc.stdin.write(f"r/2 0x{addr:08X}\n".encode())
    proc.stdin.flush()
    time.sleep(0.02)
    out = read_pty(0.05)
    m = re.search(r"0x([0-9A-Fa-f]{4,8})", out)
    if m:
        return int(m.group(1), 16) & 0xFFFF
    return None


time.sleep(1)
read_pty(0.5)

# Advance exactly 240 frames using 'n' (next instruction) or 'trace'
# mGBA's trace command can execute N instructions
# At ~280K instructions per frame, 240 frames = 67.2M instructions
# trace 67200000 would take very long...
# Better: use 'c' and set a breakpoint at VBlank handler

# Actually, mGBA doesn't have frame-level advance easily.
# Let me just run for a calculated time.
# At 60fps, 240 frames = 4 seconds. But mGBA might not run at real-time speed.
# Let me run for 10 seconds to be safe.

proc.stdin.write(b"c\n")
proc.stdin.flush()
time.sleep(10)
read_pty(1)

import signal

proc.send_signal(signal.SIGINT)
time.sleep(0.5)
read_pty(0.5)

# Check BG0HOFS to make sure we're at a similar state
bg0hofs = read_halfword(0x04000010)
print(f"BG0HOFS = {bg0hofs}")

# Read BG0 screen entries row 0
print("\nBG0 screen entries row 0 at 0xC000:")
for i in range(32):
    entry = read_halfword(0x0600C000 + i * 2)
    if entry is not None:
        tile = entry & 0x3FF
        pal = (entry >> 12) & 0xF
        print(f"  [{i:2}] raw=0x{entry:04X} tile={tile} pal={pal}")

# Read BG0 screen entries row 1
print("\nBG0 screen entries row 1:")
for i in range(32):
    entry = read_halfword(0x0600C000 + (32 + i) * 2)
    if entry is not None:
        tile = entry & 0x3FF
        pal = (entry >> 12) & 0xF
        print(f"  [{i:2}] raw=0x{entry:04X} tile={tile} pal={pal}")

# Check DISPCNT and BG0CNT
dispcnt = read_halfword(0x04000000)
bg0cnt = read_halfword(0x04000008)
print(f"\nDISPCNT = 0x{dispcnt:04X}" if dispcnt is not None else "")
print(f"BG0CNT = 0x{bg0cnt:04X}" if bg0cnt is not None else "")

proc.stdin.write(b"quit\n")
proc.stdin.flush()
try:
    proc.wait(timeout=5)
except:
    proc.kill()
os.close(master_fd)

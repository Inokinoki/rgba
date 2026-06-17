#!/usr/bin/env python3
"""Compare full EWRAM content between our emulator and mGBA"""

import subprocess
import time
import select
import os
import pty
import re
import sys

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


def read_word(addr):
    proc.stdin.write(f"r/4 0x{addr:08X}\n".encode())
    proc.stdin.flush()
    time.sleep(0.01)
    out = read_pty(0.03)
    m = re.search(r"0x([0-9A-Fa-f]{8})", out)
    if m:
        return int(m.group(1), 16)
    return None


def read_half(addr):
    proc.stdin.write(f"r/2 0x{addr:08X}\n".encode())
    proc.stdin.flush()
    time.sleep(0.01)
    out = read_pty(0.03)
    m = re.search(r"0x([0-9A-Fa-f]{4})", out)
    if m:
        return int(m.group(1), 16)
    return None


time.sleep(1)
read_pty(0.5)

# Run 5 seconds
proc.stdin.write(b"c\n")
proc.stdin.flush()
time.sleep(5)
read_pty(1)

import signal

proc.send_signal(signal.SIGINT)
time.sleep(0.5)
read_pty(0.5)

# Read full EWRAM 0x02008000-0x0200A000 in 256-word blocks
print("=== mGBA EWRAM 0x02008000-0x0200A000 ===")
mgba_ewram = {}
for base in range(0x02008000, 0x0200A000, 4):
    w = read_word(base)
    if w is not None:
        mgba_ewram[base] = w

# Find non-zero regions
nz_regions = []
current_start = None
for addr in sorted(mgba_ewram.keys()):
    if mgba_ewram[addr] != 0:
        if current_start is None:
            current_start = addr
    else:
        if current_start is not None:
            nz_regions.append((current_start, addr))
            current_start = None
if current_start:
    nz_regions.append((current_start, 0x0200A000))

print(f"Non-zero regions ({len(nz_regions)}):")
for start, end in nz_regions:
    size = end - start
    print(f"  {start:08X}-{end:08X}: {size} bytes ({size // 4} words)")

# Show first/last few words of each region
for start, end in nz_regions:
    if end - start <= 32:
        print(f"  Region {start:08X}:")
        for addr in range(start, end, 4):
            print(f"    {addr:08X}: {mgba_ewram[addr]:08X}")

proc.stdin.write(b"quit\n")
proc.stdin.flush()
try:
    proc.wait(timeout=5)
except:
    proc.kill()
os.close(master_fd)

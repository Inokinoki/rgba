#!/usr/bin/env python3
"""Use mGBA debugger to read VRAM tiles with correct output parsing"""

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


time.sleep(1)
read_pty(0.5)

# First, test reading a single word to see the output format
proc.stdin.write(b"r/4 0x06000000\n")
proc.stdin.flush()
time.sleep(0.3)
out = read_pty(0.3)
print(f"Raw r/4 output: {repr(out)}")

# Read another address
proc.stdin.write(b"r/4 0x06000004\n")
proc.stdin.flush()
time.sleep(0.3)
out = read_pty(0.3)
print(f"Raw r/4 output: {repr(out)}")

# Continue to let the game run
proc.stdin.write(b"c\n")
proc.stdin.flush()
time.sleep(8)  # Run ~8 seconds = ~480 frames
read_pty(1)  # Drain

# Break
import signal

proc.send_signal(signal.SIGINT)
time.sleep(0.5)
out = read_pty(0.5)
print(f"\nAfter break: {out[:300]}")

# Test read format again
proc.stdin.write(b"r/4 0x06000000\n")
proc.stdin.flush()
time.sleep(0.3)
out = read_pty(0.3)
print(f"\nRaw r/4 after run: {repr(out)}")

# Read tile 394 specifically (our problematic tile)
print("\nReading tile 394 from mGBA:")
addr = 0x06000000 + 394 * 32
proc.stdin.write(f"r/4 0x{addr:08X}\n".encode())
proc.stdin.flush()
time.sleep(0.3)
out = read_pty(0.3)
print(f"  Tile 394 word 0: {repr(out)}")

for i in range(1, 8):
    proc.stdin.write(f"r/4 0x{addr + i * 4:08X}\n".encode())
    proc.stdin.flush()
    time.sleep(0.1)
    out = read_pty(0.1)
    print(f"  Tile 394 word {i}: {repr(out)}")

# Read tile 0 for comparison
print("\nReading tile 0 from mGBA:")
addr = 0x06000000
for i in range(8):
    proc.stdin.write(f"r/4 0x{addr + i * 4:08X}\n".encode())
    proc.stdin.flush()
    time.sleep(0.1)
    out = read_pty(0.1)
    print(f"  Tile 0 word {i}: {repr(out)}")

proc.stdin.write(b"quit\n")
proc.stdin.flush()
try:
    proc.wait(timeout=5)
except:
    proc.kill()
os.close(master_fd)

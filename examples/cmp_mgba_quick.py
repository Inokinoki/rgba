#!/usr/bin/env python3
"""Quick check: mGBA tiles 340-480 and 0-10"""

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


def read_word(addr):
    proc.stdin.write(f"r/4 0x{addr:08X}\n".encode())
    proc.stdin.flush()
    time.sleep(0.02)
    out = read_pty(0.05)
    m = re.search(r"0x([0-9A-Fa-f]{8})", out)
    if m:
        return int(m.group(1), 16)
    return None


time.sleep(1)
read_pty(0.5)

# Run for 5 seconds
proc.stdin.write(b"c\n")
proc.stdin.flush()
time.sleep(5)
read_pty(1)

import signal

proc.send_signal(signal.SIGINT)
time.sleep(0.5)
read_pty(0.5)

# Quick tile check - just first word of each tile
print("=== mGBA tile data (first word only) ===")

# Tiles 340-480
for tid in range(340, 480):
    addr = 0x06000000 + tid * 32
    w = read_word(addr)
    status = "DATA" if w and w != 0 else "zero"
    if w and w != 0:
        print(f"  tile {tid}: 0x{w:08X} (HAS DATA)")

# Also check tiles 0-20
print("\nTiles 0-20:")
for tid in range(0, 21):
    addr = 0x06000000 + tid * 32
    w = read_word(addr)
    if w and w != 0:
        print(f"  tile {tid}: 0x{w:08X} (HAS DATA)")

proc.stdin.write(b"quit\n")
proc.stdin.flush()
try:
    proc.wait(timeout=5)
except:
    proc.kill()
os.close(master_fd)

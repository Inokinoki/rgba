#!/usr/bin/env python3
"""Compare EWRAM at 0x0200871C between our emulator and mGBA at frame 192"""

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

# Run 5 seconds (roughly 300 frames at 60fps)
proc.stdin.write(b"c\n")
proc.stdin.flush()
time.sleep(5)
read_pty(1)

import signal

proc.send_signal(signal.SIGINT)
time.sleep(0.5)
read_pty(0.5)

print("=== mGBA EWRAM at 0x0200871C (512 bytes) ===")
nonzero = 0
for i in range(128):
    addr = 0x0200871C + i * 4
    w = read_word(addr)
    if w is not None and w != 0:
        nonzero += 1
        print(f"  [{i:03d}] {addr:08X}: {w:08X}")
    elif i < 10:
        print(f"  [{i:03d}] {addr:08X}: {w if w is not None else 0:08X}")

print(f"\nNon-zero words: {nonzero}/128")

# Also check a broader range of EWRAM
print("\n=== mGBA EWRAM overview (check for data regions) ===")
for base in range(0x02008000, 0x0200A000, 0x100):
    w = read_word(base)
    if w is not None and w != 0:
        print(f"  {base:08X}: {w:08X} *")

proc.stdin.write(b"quit\n")
proc.stdin.flush()
try:
    proc.wait(timeout=5)
except:
    proc.kill()
os.close(master_fd)

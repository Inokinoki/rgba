#!/usr/bin/env python3
"""Use PTY to interact with mGBA debugger"""

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

# Create a PTY for mGBA's terminal output
master_fd, slave_fd = pty.openpty()

proc = subprocess.Popen(
    ["mgba", "-d", ROM],
    stdin=subprocess.PIPE,
    stdout=slave_fd,
    stderr=slave_fd,
    env=env,
)

os.close(slave_fd)  # Close slave in parent


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
    return buf


time.sleep(2)
initial = read_pty(1)
print("=== Initial ===")
print(initial.decode(errors="replace")[:1000])

# Send command
proc.stdin.write(b"help\n")
proc.stdin.flush()
time.sleep(1)
out = read_pty(0.5)
print("\n=== Help ===")
print(out.decode(errors="replace")[:2000])

# Try reading memory
proc.stdin.write(b"x/4w 0x06000000\n")
proc.stdin.flush()
time.sleep(1)
out = read_pty(0.5)
print("\n=== x/4w 0x06000000 ===")
print(out.decode(errors="replace")[:500])

proc.stdin.write(b"quit\n")
proc.stdin.flush()
try:
    proc.wait(timeout=5)
except:
    proc.kill()
os.close(master_fd)

#!/usr/bin/env python3
"""Debug mGBA debugger output format"""

import subprocess
import time
import select
import os

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
    while select.select([proc.stdout], [], [], 0.3)[0]:
        chunk = proc.stdout.read1(65536)
        if not chunk:
            break
        buf += chunk
    return buf


time.sleep(1)
initial = drain()
print("=== Initial output ===")
print(initial.decode(errors="replace")[:1000])

# Try commands
proc.stdin.write(b"regs\n")
proc.stdin.flush()
time.sleep(0.5)
out = drain()
print("\n=== After 'regs' ===")
print(out.decode(errors="replace")[:500])

# Try reading memory
proc.stdin.write(b"x/4w 0x06000000\n")
proc.stdin.flush()
time.sleep(0.5)
out = drain()
print("\n=== After 'x/4w 0x06000000' ===")
print(out.decode(errors="replace")[:500])

# Try advance one frame
proc.stdin.write(b"frame\n")
proc.stdin.flush()
time.sleep(0.5)
out = drain()
print("\n=== After 'frame' ===")
print(out.decode(errors="replace")[:500])

# Read VRAM again
proc.stdin.write(b"x/4w 0x06000000\n")
proc.stdin.flush()
time.sleep(0.5)
out = drain()
print("\n=== VRAM after 1 frame ===")
print(out.decode(errors="replace")[:500])

# Try help
proc.stdin.write(b"help\n")
proc.stdin.flush()
time.sleep(0.5)
out = drain()
print("\n=== Help ===")
print(out.decode(errors="replace")[:1000])

proc.stdin.write(b"quit\n")
proc.stdin.flush()
try:
    proc.wait(timeout=5)
except:
    proc.kill()

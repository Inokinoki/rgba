#!/usr/bin/env python3
"""Simple mGBA debugger script to dump VRAM"""

import subprocess
import time
import re
import struct
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

time.sleep(2)

# Read initial output
import select

initial = b""
while select.select([proc.stdout], [], [], 0.1)[0]:
    chunk = proc.stdout.read1(4096)
    if not chunk:
        break
    initial += chunk
print("Initial output:", initial.decode(errors="replace")[:500])

# Advance frames using debugger commands
for i in range(240):
    proc.stdin.write(b"frame\n")
    proc.stdin.flush()

time.sleep(3)

# Read buffered output
buf = b""
while select.select([proc.stdout], [], [], 0.5)[0]:
    chunk = proc.stdout.read1(65536)
    if not chunk:
        break
    buf += chunk
print(f"Buffered output after frames: {len(buf)} bytes")

# Now try reading VRAM - try the 'x' command
proc.stdin.write(b"x/4w 0x06000000\n")
proc.stdin.flush()
time.sleep(0.5)

while select.select([proc.stdout], [], [], 0.5)[0]:
    chunk = proc.stdout.read1(65536)
    if not chunk:
        break
    buf += chunk
print("After x command:", buf.decode(errors="replace")[-500:])

# Try save state approach
proc.stdin.write(b"save /tmp/mgba_state.ss0\n")
proc.stdin.flush()
time.sleep(1)

while select.select([proc.stdout], [], [], 0.5)[0]:
    chunk = proc.stdout.read1(65536)
    if not chunk:
        break
    buf += chunk

# Quit
proc.stdin.write(b"quit\n")
proc.stdin.flush()
try:
    proc.wait(timeout=5)
except:
    proc.kill()

# Check if save state was created
if os.path.exists("/tmp/mgba_state.ss0"):
    print(f"Save state created: {os.path.getsize('/tmp/mgba_state.ss0')} bytes")
else:
    print("No save state created")

#!/usr/bin/env python3
"""Check mGBA output channels"""

import subprocess
import time
import select
import os

ROM = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba"
env = os.environ.copy()
env["SDL_VIDEODRIVER"] = "dummy"
env["SDL_AUDIODRIVER"] = "dummy"
env["PATH"] = "/usr/games:" + env.get("PATH", "")

# Try with separate stderr
proc = subprocess.Popen(
    ["mgba", "-d", ROM],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    env=env,
)

time.sleep(2)

proc.stdin.write(b"help\n")
proc.stdin.flush()
time.sleep(1)

# Check both stdout and stderr
stdout_data = b""
stderr_data = b""
while select.select([proc.stdout], [], [], 0.1)[0]:
    stdout_data += proc.stdout.read1(65536)
while select.select([proc.stderr], [], [], 0.1)[0]:
    stderr_data += proc.stderr.read1(65536)

print(f"stdout: {len(stdout_data)} bytes: {stdout_data.decode(errors='replace')[:500]}")
print(f"stderr: {len(stderr_data)} bytes: {stderr_data.decode(errors='replace')[:500]}")

# Maybe mGBA outputs to the terminal directly (not piped)?
# Let me try using a pty
proc.stdin.write(b"quit\n")
proc.stdin.flush()
try:
    proc.wait(timeout=5)
except:
    proc.kill()

#!/usr/bin/env python3
"""Use mGBA debugger to find what writes to EWRAM 0x0200883C (palette data)"""

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
    return buf.decode(errors="replace")


def send_cmd(cmd):
    proc.stdin.write(f"{cmd}\n".encode())
    proc.stdin.flush()


time.sleep(1)
read_pty(0.5)

# Set a write breakpoint at the palette data location
send_cmd("bw 0x0200883C")
time.sleep(0.3)
read_pty(0.3)

# Continue and let it hit the breakpoint
send_cmd("c")
time.sleep(3)

import signal

proc.send_signal(signal.SIGINT)
time.sleep(0.5)
out = read_pty(1)
print("=== Breakpoint hit output ===")
print(out[-2000:] if len(out) > 2000 else out)

# Try continuing a few more times
for i in range(5):
    send_cmd("c")
    time.sleep(1)
    proc.send_signal(signal.SIGINT)
    time.sleep(0.3)
    out = read_pty(0.5)
    # Look for breakpoint hit or PC info
    lines = out.strip().split("\n")
    for line in lines[-10:]:
        if (
            "0200883C" in line.upper()
            or "BREAK" in line.upper()
            or "PC" in line.upper()
        ):
            print(f"  Hit {i}: {line.strip()}")

send_cmd("quit")
proc.stdin.flush()
try:
    proc.wait(timeout=5)
except:
    proc.kill()
os.close(master_fd)

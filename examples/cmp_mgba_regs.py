#!/usr/bin/env python3
"""Take screenshot from mGBA and save as PPM"""

import subprocess
import time
import select
import os
import pty
import re
import struct

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

# Run for 5 seconds (~300 frames)
proc.stdin.write(b"c\n")
proc.stdin.flush()
time.sleep(5)
read_pty(1)

import signal

proc.send_signal(signal.SIGINT)
time.sleep(0.5)
read_pty(0.5)

# Read palette RAM (0x05000000-0x050003FF, 512 bytes = 256 colors)
print("=== Palette RAM ===")
palette = bytearray(512)
for i in range(0, 512, 4):
    addr = 0x05000000 + i
    w = read_word(addr)
    if w is not None:
        struct.pack_into("<I", palette, i, w)

# Show first 16 palette colors
print("First 16 BG palette colors:")
for i in range(16):
    color = struct.unpack_from("<H", palette, i * 2)[0]
    r = color & 0x1F
    g = (color >> 5) & 0x1F
    b = (color >> 10) & 0x1F
    print(f"  [{i}] = 0x{color:04X} (R={r}, G={g}, B={b})")

# Read DISPCNT and BGCNT
print("\n=== IO Registers ===")
dispcnt = read_word(0x04000000)
print(f"DISPCNT = 0x{dispcnt:08X}")
bg0cnt = read_word(0x04000008)
print(f"BG0CNT = 0x{bg0cnt:08X}")

# Read BG0HOFS, BG0VOFS
bg0hofs = read_word(0x04000010)
bg0vofs = read_word(0x04000012)
print(f"BG0HOFS = 0x{bg0hofs:08X}")
print(f"BG0VOFS = 0x{bg0vofs:08X}")

# Read BLDCNT, BLDALPHA, BLDY
bldcnt = read_word(0x04000050)
bldalpha = read_word(0x04000052)
bldy = read_word(0x04000054)
print(f"BLDCNT = 0x{bldcnt:08X}")
print(f"BLDALPHA = 0x{bldalpha:08X}")
print(f"BLDY = 0x{bldy:08X}")

# Read backdrop color (first palette entry)
backdrop = read_word(0x05000000)
print(f"Backdrop = 0x{backdrop:08X}")

proc.stdin.write(b"quit\n")
proc.stdin.flush()
try:
    proc.wait(timeout=5)
except:
    proc.kill()
os.close(master_fd)

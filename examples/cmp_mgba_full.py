#!/usr/bin/env python3
"""Read framebuffer from mGBA by reading VRAM in bitmap mode"""

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


def read_halfword(addr):
    proc.stdin.write(f"r/2 0x{addr:08X}\n".encode())
    proc.stdin.flush()
    time.sleep(0.02)
    out = read_pty(0.05)
    m = re.search(r"0x([0-9A-Fa-f]{4,8})", out)
    if m:
        return int(m.group(1), 16) & 0xFFFF
    return None


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

# Read key IO registers
print("=== mGBA IO registers ===")
dispcnt = read_halfword(0x04000000)
print(f"DISPCNT = 0x{dispcnt:04X}" if dispcnt is not None else "DISPCNT read failed")

bg0cnt = read_halfword(0x04000008)
bg1cnt = read_halfword(0x0400000A)
bg2cnt = read_halfword(0x0400000C)
bg3cnt = read_halfword(0x0400000E)
print(f"BG0CNT = 0x{bg0cnt:04X}" if bg0cnt is not None else "BG0CNT read failed")
print(f"BG1CNT = 0x{bg1cnt:04X}" if bg1cnt is not None else "BG1CNT read failed")
print(f"BG2CNT = 0x{bg2cnt:04X}" if bg2cnt is not None else "BG2CNT read failed")
print(f"BG3CNT = 0x{bg3cnt:04X}" if bg3cnt is not None else "BG3CNT read failed")

bg0hofs = read_halfword(0x04000010)
bg0vofs = read_halfword(0x04000012)
print(f"BG0HOFS = {bg0hofs}" if bg0hofs is not None else "BG0HOFS read failed")
print(f"BG0VOFS = {bg0vofs}" if bg0vofs is not None else "BG0VOFS read failed")

bldcnt = read_halfword(0x04000050)
bldalpha = read_halfword(0x04000052)
bldy = read_halfword(0x04000054)
print(f"BLDCNT = 0x{bldcnt:04X}" if bldcnt is not None else "BLDCNT read failed")
print(
    f"BLDALPHA = 0x{bldalpha:04X}" if bldalpha is not None else "BLDALPHA read failed"
)
print(f"BLDY = {bldy}" if bldy is not None else "BLDY read failed")

# Read palette
print("\n=== mGBA BG palette ===")
for i in range(16):
    color = read_halfword(0x05000000 + i * 2)
    if color is not None:
        r = color & 0x1F
        g = (color >> 5) & 0x1F
        b = (color >> 10) & 0x1F
        print(f"  [{i}] = 0x{color:04X} (R={r}, G={g}, B={b})")

# Read a few specific tile data bytes
print("\n=== mGBA tile 277 data (has data in our emu too) ===")
for i in range(8):
    w = read_word(0x06000000 + 277 * 32 + i * 4)
    print(f"  word {i}: 0x{w:08X}" if w is not None else f"  word {i}: FAILED")

print("\n=== mGBA tile 0 data ===")
for i in range(8):
    w = read_word(0x06000000 + i * 4)
    print(f"  word {i}: 0x{w:08X}" if w is not None else f"  word {i}: FAILED")

# Read screen entries
print("\n=== mGBA BG0 screen entries at 0xC000 ===")
for i in range(0, 64, 1):
    addr = 0x0600C000 + i * 2
    entry = read_halfword(addr)
    if entry is not None:
        tile = entry & 0x3FF
        if i % 32 == 0:
            print(f"\n  row {i // 32}:")
        print(f"    [{i % 32}] tile={tile} raw=0x{entry:04X}")

proc.stdin.write(b"quit\n")
proc.stdin.flush()
try:
    proc.wait(timeout=5)
except:
    proc.kill()
os.close(master_fd)

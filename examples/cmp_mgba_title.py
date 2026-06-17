#!/usr/bin/env python3
"""Compare VRAM, palette, and BG registers with mGBA at the title screen (frame 424)"""

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


def mgba_cmd(cmd, wait=0.1):
    proc.stdin.write((cmd + "\n").encode())
    proc.stdin.flush()
    time.sleep(wait)
    return read_pty(0.1)


def read_word(addr):
    out = mgba_cmd(f"r/4 0x{addr:08X}", 0.05)
    m = re.search(r"=\s*0x([0-9A-Fa-f]{8})", out)
    if m:
        return int(m.group(1), 16)
    m = re.search(r"([0-9A-Fa-f]{8})\s*=\s*0x([0-9A-Fa-f]{8})", out)
    if m:
        return int(m.group(2), 16)
    return None


def read_half(addr):
    out = mgba_cmd(f"r/2 0x{addr:08X}", 0.05)
    m = re.search(r"=\s*0x([0-9A-Fa-f]{4})", out)
    if m:
        return int(m.group(1), 16)
    m = re.search(r"([0-9A-Fa-f]{8})\s*=\s*0x([0-9A-Fa-f]{4})", out)
    if m:
        return int(m.group(2), 16)
    return None


def read_block(addr, size):
    result = bytearray(size)
    for i in range(0, size, 4):
        w = read_word(addr + i)
        if w is None:
            continue
        result[i : i + 4] = struct.pack("<I", w)
    return bytes(result)


time.sleep(1.5)
read_pty(0.5)

# We need to run mGBA to frame 424 (240 frames boot, START press 4 frames, release 180 frames)
# mGBA at 60fps runs ~60 frames/sec. So 424/60 ≈ 7 seconds.
# Use continue to run, then break after delay
mgba_cmd("c", 0.1)  # start running
time.sleep(4.0)  # ~240 frames

# Press START - we can't directly control mGBA input via debugger
# So we'll just compare at the title screen (240 frames, before pressing START)
# Actually let's just let it run and break

import signal

proc.send_signal(signal.SIGINT)
time.sleep(0.5)
out = read_pty(0.5)
print(f"After first break: {out[:200]}")

# Read BG registers from mGBA
# IO registers at 0x04000000
dispcnt = read_half(0x04000000)
bg0cnt = read_half(0x04000008)
bg1cnt = read_half(0x0400000A)
bg2cnt = read_half(0x0400000C)
bg3cnt = read_half(0x0400000E)

print(f"\nmGBA DISPCNT: {dispcnt:04X}")
print(f"mGBA BG0CNT: {bg0cnt:04X}")
print(f"mGBA BG1CNT: {bg1cnt:04X}")
print(f"mGBA BG2CNT: {bg2cnt:04X}")
print(f"mGBA BG3CNT: {bg3cnt:04X}")

for i, cnt in enumerate([bg0cnt, bg1cnt, bg2cnt, bg3cnt]):
    priority = cnt & 3
    char_base = ((cnt >> 2) & 3) * 0x4000
    palette_mode = "256-color" if (cnt >> 7) & 1 else "16-color"
    screen_base = ((cnt >> 8) & 0x1F) * 0x800
    size = (cnt >> 14) & 3
    print(
        f"  mGBA BG{i}: pri={priority} char_base={0x06000000 + char_base:05X} screen_base={0x06000000 + screen_base:05X} {palette_mode} size={size}"
    )

# Read palette (first 512 bytes = 256 entries)
print("\n=== mGBA BG PALETTE (non-zero) ===")
pal_data = read_block(0x05000000, 0x200)
for i in range(256):
    c = struct.unpack_from("<H", pal_data, i * 2)[0]
    if c != 0:
        r = c & 0x1F
        g = (c >> 5) & 0x1F
        b = (c >> 10) & 0x1F
        if i < 32:
            print(f"  PAL[{i}] = {c:04X} (R{r} G{g} B{b})")

# Read BG0 screen entries (1024 entries)
print("\n=== mGBA BG0 screen entries (first 40 non-zero) ===")
if bg0cnt:
    screen_base = ((bg0cnt >> 8) & 0x1F) * 0x800
    se_data = read_block(0x06000000 + screen_base, 2048)
    count = 0
    for i in range(1024):
        entry = struct.unpack_from("<H", se_data, i * 2)[0]
        if entry != 0 and count < 40:
            tile = entry & 0x3FF
            hflip = (entry >> 10) & 1
            vflip = (entry >> 11) & 1
            pal = (entry >> 12) & 0xF
            print(
                f"  SE[{i}] = {entry:04X} (tile={tile} h={hflip} v={vflip} pal={pal})"
            )
            count += 1

# Read tile 1023 data
print("\n=== mGBA Tile 1023 data ===")
tile_data = read_block(0x06000000 + 1023 * 32, 32)
print(f"  {' '.join(f'{b:02X}' for b in tile_data)}")

# Check how many tiles have data at char_base 0
print("\n=== mGBA tiles with data at 0x6000000 ===")
tile_block = read_block(0x06000000, 0x8000)  # first 1024 tiles
tiles_with_data = 0
for t in range(1024):
    has_data = any(tile_block[t * 32 + t * 32 + 32])
    if any(tile_block[t * 32 + j] for j in range(32)):
        tiles_with_data += 1
print(f"  {tiles_with_data} tiles with data (of 1024)")

proc.stdin.write(b"quit\n")
proc.stdin.flush()
try:
    proc.wait(timeout=5)
except:
    proc.kill()
os.close(master_fd)

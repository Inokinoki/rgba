#!/usr/bin/env python3
"""
Quick check of VRAM BG0 screen entries and IWRAM in mGBA to verify game state.
"""

import pty, os, select, time, subprocess

ROM = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba"


def ru(fd, marker, timeout=30):
    buf = b""
    t0 = time.time()
    while time.time() - t0 < timeout:
        r, _, _ = select.select([fd], [], [], 0.05)
        if r:
            try:
                d = os.read(fd, 4096)
                if not d:
                    break
                buf += d
                if marker in buf:
                    return buf
            except:
                break
    return buf


def sc(fd, cmd):
    os.write(fd, (cmd + "\n").encode())


def read_hw(fd, addr):
    sc(fd, f"r/2 0x{addr:08X}")
    buf = ru(fd, b">", timeout=3)
    text = buf.decode(errors="replace")
    for line in text.split("\n"):
        line = line.strip()
        if "=" in line:
            parts = line.split("=")
            if len(parts) >= 2:
                try:
                    return int(parts[-1].strip(), 16)
                except:
                    pass
    return 0xFFFF


def read_word(fd, addr):
    sc(fd, f"r/4 0x{addr:08X}")
    buf = ru(fd, b">", timeout=3)
    text = buf.decode(errors="replace")
    for line in text.split("\n"):
        line = line.strip()
        if "=" in line:
            parts = line.split("=")
            if len(parts) >= 2:
                try:
                    return int(parts[-1].strip(), 16)
                except:
                    pass
    return 0xFFFFFFFF


mf, sf = pty.openpty()
env = os.environ.copy()
env["SDL_VIDEODRIVER"] = "dummy"
env["SDL_AUDIODRIVER"] = "dummy"
proc = subprocess.Popen(
    ["/usr/games/mgba", "-d", ROM], stdin=sf, stdout=sf, stderr=sf, env=env
)
os.close(sf)

print("Waiting for debugger...")
ru(mf, b">", timeout=15)
print("Ready. Advancing 240 frames...")
for i in range(240):
    sc(mf, "frame")
    if (i + 1) % 120 == 0:
        ru(mf, b">", timeout=10)
ru(mf, b">", timeout=10)

# Check DISPCNT
dispcnt = read_hw(mf, 0x04000000)
print(f"\nDISPCNT: 0x{dispcnt:04X}")
mode = dispcnt & 7
bg_en = (dispcnt >> 8) & 0xF
print(f"  Mode: {mode}, BG enable: {bg_en:04b}")

# Check BG0CNT
bg0cnt = read_hw(mf, 0x04000008)
bg0_priority = bg0cnt & 3
bg0_char_base = ((bg0cnt >> 2) & 3) * 0x4000
bg0_screen_base = ((bg0cnt >> 8) & 0x1F) * 0x800
bg0_size = (bg0cnt >> 14) & 3
print(f"\nBG0CNT: 0x{bg0cnt:04X}")
print(
    f"  Priority: {bg0_priority}, Char base: 0x{bg0_char_base:04X}, Screen base: 0x{bg0_screen_base:04X}, Size: {bg0_size}"
)

# Check VRAM at BG0 screen base
print(f"\nVRAM BG0 screen entries at 0x{0x06000000 + bg0_screen_base:08X}:")
non_zero = 0
for i in range(32):
    addr = 0x06000000 + bg0_screen_base + i * 2
    val = read_hw(mf, addr)
    tile = val & 0x3FF
    pal = (val >> 12) & 0xF
    if val != 0:
        non_zero += 1
    if i % 16 == 0:
        print(f"\n  [{i:03X}]: ", end="")
    if val == 0:
        print("   . ", end="")
    else:
        print(f"{val:04X} ", end="")
print(f"\n  Non-zero: {non_zero}/32")

# Check IWRAM more broadly - scan for non-FF ranges
print("\nIWRAM scan for non-0xFF regions (checking every 256 bytes):")
for block in range(0, 0x8000, 256):
    non_ff = 0
    for off in range(0, 256, 4):
        addr = 0x03000000 + block + off
        val = read_hw(mf, addr)
        if val != 0xFFFF:
            non_ff += 1
    if non_ff > 0:
        print(
            f"  0x{0x03000000 + block:08X}-0x{0x03000000 + block + 255:08X}: {non_ff}/64 non-0xFFFF halfwords"
        )

# Check VRAM tile data
print("\nVRAM tile data (first 64 bytes):")
for i in range(0, 64, 16):
    addr = 0x06000000 + bg0_char_base + i
    sc(mf, f"r/4 0x{addr:08X}")
    buf = ru(mf, b">", timeout=3)
    text = buf.decode(errors="replace")
    for line in text.split("\n"):
        line = line.strip()
        if "=" in line:
            print(f"  {addr:08X}: {line.strip()}")
            break

sc(mf, "quit")
time.sleep(0.5)
proc.kill()
os.close(mf)

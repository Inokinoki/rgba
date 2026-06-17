#!/usr/bin/env python3
"""
Dump IWRAM screen entries from mGBA debugger for comparison with RGBA emulator.
Reads 512 halfwords from 0x03006DD8 using mGBA's command-line debugger.
"""

import pty
import os
import select
import sys
import time
import subprocess

ROM_PATH = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba"
IWRAM_START = 0x03006DD8
NUM_ENTRIES = 512


def read_until(fd, marker, timeout=30):
    buf = b""
    start = time.time()
    while time.time() - start < timeout:
        r, _, _ = select.select([fd], [], [], 0.1)
        if r:
            try:
                data = os.read(fd, 4096)
                if not data:
                    break
                buf += data
                if marker in buf:
                    return buf
            except OSError:
                break
    return buf


def send_cmd(fd, cmd):
    os.write(fd, (cmd + "\n").encode())


def main():
    master_fd, slave_fd = pty.openpty()

    env = os.environ.copy()
    env["SDL_VIDEODRIVER"] = "dummy"
    env["SDL_AUDIODRIVER"] = "dummy"

    proc = subprocess.Popen(
        ["/usr/games/mgba", "-d", ROM_PATH],
        stdin=slave_fd,
        stdout=slave_fd,
        stderr=slave_fd,
        env=env,
    )
    os.close(slave_fd)

    print("Waiting for mGBA debugger prompt...")
    buf = read_until(master_fd, b">", timeout=15)
    if b">" not in buf:
        print("ERROR: No debugger prompt. Buf:", buf.decode(errors="replace")[:500])
        proc.kill()
        os.close(master_fd)
        return
    print("Debugger ready.")

    # Advance 240 frames using the "frame" command
    # Send in batches to speed things up
    print("Advancing 240 frames...")
    for i in range(240):
        send_cmd(master_fd, "frame")
        if (i + 1) % 60 == 0:
            buf = read_until(master_fd, b">", timeout=10)
            print(f"  Frame {i + 1}/240")

    buf = read_until(master_fd, b">", timeout=10)

    # Read 512 halfwords
    print(f"Reading {NUM_ENTRIES} halfwords from 0x{IWRAM_START:08X}...")
    entries = []

    for i in range(NUM_ENTRIES):
        addr = IWRAM_START + i * 2
        send_cmd(master_fd, f"r/2 0x{addr:08X}")
        buf = read_until(master_fd, b">", timeout=5)

        text = buf.decode(errors="replace")
        found = False
        for line in text.split("\n"):
            line = line.strip()
            if "=" in line and "0x" in line:
                parts = line.split("=")
                if len(parts) >= 2:
                    val_str = parts[-1].strip().rstrip()
                    try:
                        val = int(val_str, 16)
                        entries.append(val)
                        found = True
                        break
                    except ValueError:
                        pass
        if not found:
            entries.append(0xFFFF)

        if (i + 1) % 64 == 0:
            print(f"  Read {i + 1}/{NUM_ENTRIES}")

    # Palette distribution
    pal_counts = [0] * 16
    for entry in entries:
        pal = (entry >> 12) & 0xF
        pal_counts[pal] += 1

    print(f"\nPalette distribution ({len(entries)} entries):")
    for p, count in enumerate(pal_counts):
        if count > 0:
            print(f"  palette {p}: {count} entries")

    # Raw hex dump matching RGBA format
    print("\n=== Raw hex dump ===")
    for i in range(0, NUM_ENTRIES, 16):
        addr_off = IWRAM_START - 0x03000000 + i * 2
        line = f"{addr_off:04X}: "
        for j in range(16):
            if i + j < NUM_ENTRIES:
                line += f"{entries[i + j]:04X} "
        print(line)

    # Save for comparison
    with open("/tmp/mgba_iwram_dump.txt", "w") as f:
        for i in range(0, NUM_ENTRIES, 16):
            addr_off = IWRAM_START - 0x03000000 + i * 2
            line = f"{addr_off:04X}: "
            for j in range(16):
                if i + j < NUM_ENTRIES:
                    line += f"{entries[i + j]:04X} "
            f.write(line.strip() + "\n")
    print(f"\nSaved to /tmp/mgba_iwram_dump.txt")

    # Screen entries view
    print("\nScreen entries (tile/Ppal):")
    for i in range(0, min(NUM_ENTRIES, 64), 32):
        print(f"  [{i:03X}]: ", end="")
        for j in range(32):
            if i + j < NUM_ENTRIES:
                entry = entries[i + j]
                tile = entry & 0x3FF
                pal = (entry >> 12) & 0xF
                if pal == 0:
                    print(f"{tile:4} ", end="")
                else:
                    print(f"P{pal}{tile:3} ", end="")
        print()

    send_cmd(master_fd, "quit")
    time.sleep(0.5)
    proc.kill()
    os.close(master_fd)
    print("Done.")


if __name__ == "__main__":
    main()
